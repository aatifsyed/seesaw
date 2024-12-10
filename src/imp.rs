use std::{
    borrow::Cow,
    env,
    error::Error,
    fmt,
    io::{self, Write as _},
    ops::BitAnd,
    path::PathBuf,
    process::{Command, Stdio},
    str,
};

use proc_macro2::Span;
use quote::ToTokens as _;
use regex::RegexSet;
use syn::{
    parse_quote, punctuated::Punctuated, token, visit::Visit, Abi, ForeignItemFn, Generics, Item,
    ItemTrait, LitStr, Token, TraitItem, TraitItemFn, Visibility,
};

/// Generate a trait from a `C` header that's passed through [`bindgen`](https://docs.rs/bindgen).
///
/// Returns an error if:
/// - Invalid names or regexes were passed to the constituent [`Trait`]s.
/// - The `bindings` couldn't be parsed as a Rust file.
/// - There was an error writing to `dest`.
///
/// See [crate documentation](crate) for more.
pub fn seesaw<'a>(
    traits: impl Into<TraitSet>,
    bindings: impl fmt::Display,
    dest: impl Into<Destination<'a>>,
) -> io::Result<()> {
    let items = _seesaw(traits.into(), bindings.to_string())?;
    let file = syn::File {
        shebang: None,
        attrs: vec![],
        items: items.into_iter().map(Item::Trait).collect(),
    };

    let mut rustfmt = match env::var_os("RUSTFMT") {
        Some(it) => Command::new(it),
        None => Command::new("rustfmt"),
    };

    let rustfmt = match rustfmt
        .stdin(Stdio::piped())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            let mut stdin = child.stdin.take().unwrap();
            let ts = file.to_token_stream();
            match fmt::write(&mut Write2Write(&mut stdin), format_args!("{ts}")).is_err()
                || stdin.flush().is_err()
            {
                true => None,
                false => {
                    drop(stdin);
                    match child.wait_with_output() {
                        Ok(out) if out.status.success() => Some(out.stdout),
                        _ => None,
                    }
                }
            }
        }
        Err(_) => None,
    };
    let formatted = rustfmt.unwrap_or_else(|| Vec::from(prettyplease::unparse(&file)));

    let mut writer = match dest.into() {
        Destination::Writer(write) => write,
        Destination::Path(it) => Box::new(std::fs::File::create(it)?),
    };
    writeln!(writer, "/* this file is @generated by seesaw */\n")?;
    io::copy(&mut &formatted[..], &mut writer)?;
    Ok(())
}

/// Utility struct for where bindings are written.
///
/// Implements [`From<Path>`](std::path::Path) etc.
pub enum Destination<'a> {
    Path(Cow<'a, std::path::Path>),
    Writer(Box<dyn io::Write + 'a>),
}
impl<'a> From<&'a std::path::Path> for Destination<'a> {
    fn from(value: &'a std::path::Path) -> Self {
        Self::Path(Cow::Borrowed(value))
    }
}
impl From<PathBuf> for Destination<'_> {
    fn from(value: PathBuf) -> Self {
        Self::Path(Cow::Owned(value))
    }
}
impl<'a> From<&'a str> for Destination<'a> {
    fn from(value: &'a str) -> Self {
        Self::from(std::path::Path::new(value))
    }
}
impl From<String> for Destination<'_> {
    fn from(value: String) -> Self {
        Self::from(PathBuf::from(value))
    }
}
impl<'a> From<&'a mut Vec<u8>> for Destination<'a> {
    fn from(value: &'a mut Vec<u8>) -> Self {
        Self::Writer(Box::new(value))
    }
}

impl<'a> From<&'a mut String> for Destination<'a> {
    fn from(value: &'a mut String) -> Self {
        Self::Writer(Box::new(Write2Write(value)))
    }
}

fn _seesaw(TraitSet(traits): TraitSet, bindings: String) -> io::Result<Vec<ItemTrait>> {
    let bindings = err(
        io::ErrorKind::InvalidData,
        syn::parse_file(&bindings.to_string()),
    )?;

    let span = Span::call_site();

    traits
        .into_iter()
        .map(
            |Trait {
                 name,
                 allowlist,
                 blocklist,
             }| {
                Ok(ItemTrait {
                    attrs: vec![parse_quote!(#[allow(unused)])],
                    vis: Visibility::Inherited,
                    unsafety: None,
                    auto_token: None,
                    restriction: None,
                    trait_token: Token![trait](span),
                    ident: err(io::ErrorKind::InvalidInput, syn::parse_str(&name))?,
                    generics: Generics::default(),
                    colon_token: None,
                    supertraits: Punctuated::new(),
                    brace_token: token::Brace(span),
                    items: extract(
                        &err(io::ErrorKind::InvalidInput, RegexSet::new(allowlist))?,
                        &err(io::ErrorKind::InvalidInput, RegexSet::new(blocklist))?,
                        &bindings,
                    )
                    .into_iter()
                    .map(|it| {
                        let mut sig = it.sig.clone();
                        sig.unsafety = Some(Token![unsafe](span));
                        sig.abi = Some(Abi {
                            extern_token: Token![extern](span),
                            name: Some(LitStr::new("C", span)),
                        });
                        TraitItem::Fn(TraitItemFn {
                            attrs: it.attrs.clone(),
                            sig,
                            default: None,
                            semi_token: Some(it.semi_token),
                        })
                    })
                    .collect(),
                })
            },
        )
        .collect()
}

/// A specification of a `trait` generated from a `C` header.
///
/// You can [`allow`](Self::allow) and [`block`](Self::block) functions for inclusion.
#[derive(Debug, Clone)]
pub struct Trait {
    name: String,
    allowlist: Vec<String>,
    blocklist: Vec<String>,
}

impl Trait {
    /// The name of the trait.
    ///
    /// This SHOULD be a valid Rust identifier.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            allowlist: vec![],
            blocklist: vec![],
        }
    }
    /// Include functions that match this regex in the trait.
    ///
    /// If this is never called, all functions are included by default.
    pub fn allow(self, s: impl Into<String>) -> Self {
        self.allow_all([s])
    }
    /// Equivalent to calling [`allow`](Self::allow) multiple times.
    pub fn allow_all<S: Into<String>>(mut self, i: impl IntoIterator<Item = S>) -> Self {
        self.allowlist.extend(i.into_iter().map(Into::into));
        self
    }

    /// Exclude functions that match this regex from the trait.
    pub fn block(self, s: impl Into<String>) -> Self {
        self.block_all([s])
    }
    /// Equivalent to calling [`block`](Self::block) multiple times.
    pub fn block_all<S: Into<String>>(mut self, i: impl IntoIterator<Item = S>) -> Self {
        self.blocklist.extend(i.into_iter().map(Into::into));
        self
    }
}

impl BitAnd<Self> for Trait {
    type Output = TraitSet;
    fn bitand(self, rhs: Self) -> Self::Output {
        TraitSet(vec![self, rhs])
    }
}

impl BitAnd<TraitSet> for Trait {
    type Output = TraitSet;
    fn bitand(self, rhs: TraitSet) -> Self::Output {
        rhs & self
    }
}

/// A combination of multiple [`Trait`] definitions.
#[derive(Debug, Default, Clone)]
pub struct TraitSet(Vec<Trait>);

impl TraitSet {
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<Trait> for TraitSet {
    fn from(value: Trait) -> Self {
        Self(vec![value])
    }
}

impl From<String> for TraitSet {
    fn from(value: String) -> Self {
        Trait::new(value).into()
    }
}
impl From<&str> for TraitSet {
    fn from(value: &str) -> Self {
        Self::from(String::from(value))
    }
}

impl BitAnd<Trait> for TraitSet {
    type Output = Self;
    fn bitand(mut self, rhs: Trait) -> Self::Output {
        self.0.push(rhs);
        self
    }
}

impl BitAnd<Self> for TraitSet {
    type Output = Self;
    fn bitand(mut self, mut rhs: Self) -> Self::Output {
        self.0.append(&mut rhs.0);
        self
    }
}

impl Extend<Trait> for TraitSet {
    fn extend<T: IntoIterator<Item = Trait>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl FromIterator<Trait> for TraitSet {
    fn from_iter<T: IntoIterator<Item = Trait>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

fn extract<'ast>(
    allowlist: &RegexSet,
    blocklist: &RegexSet,
    file: &'ast syn::File,
) -> Vec<&'ast ForeignItemFn> {
    struct Visitor<'a, 'ast> {
        allowlist: &'a RegexSet,
        blocklist: &'a RegexSet,
        selected: Vec<&'ast ForeignItemFn>,
    }
    impl<'ast> Visit<'ast> for Visitor<'_, 'ast> {
        fn visit_foreign_item_fn(&mut self, i: &'ast ForeignItemFn) {
            if allowed(self.allowlist, self.blocklist, &i.sig.ident.to_string()) {
                self.selected.push(i)
            };
        }
    }
    let mut visitor = Visitor {
        allowlist,
        blocklist,
        selected: vec![],
    };

    visitor.visit_file(file);

    visitor.selected
}

fn allowed(allowlist: &RegexSet, blocklist: &RegexSet, s: &str) -> bool {
    match (
        allowlist.is_empty(),
        allowlist.is_match(s),
        blocklist.is_empty(),
        blocklist.is_match(s),
    ) {
        (_, _, false, true) => false,  // explicit block
        (false, true, _, _) => true,   // explicit allow
        (false, false, _, _) => false, // not allowed
        (true, _, _, _) => true,       // allow by default
    }
}

#[test]
fn test_allowed() {
    #[track_caller]
    fn t(allow: &[&str], block: &[&str], s: &str, expected: bool) {
        let allow = &RegexSet::new(allow).unwrap();
        let block = &RegexSet::new(block).unwrap();
        assert_eq!(
            allowed(allow, block, s),
            expected,
            "allow={allow:?}, block={block:?} on {s}"
        )
    }
    t(&[], &[], "hello", true);
    t(&[], &["hello"], "hello", false);
    t(&["hello"], &["goodbye"], "hello", true);
}

fn err<T>(
    kind: io::ErrorKind,
    res: Result<T, impl Error + Send + Sync + 'static>,
) -> io::Result<T> {
    match res {
        Ok(it) => Ok(it),
        Err(e) => Err(io::Error::new(kind, e)),
    }
}

struct Write2Write<T>(T);

/// You must remember to call [`io::Write::flush`] appropriately.
impl<T: io::Write> fmt::Write for Write2Write<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        self.0.write_fmt(args).map_err(|_| fmt::Error)
    }
}

impl<T: fmt::Write> io::Write for Write2Write<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self
            .0
            .write_str(err(io::ErrorKind::InvalidData, str::from_utf8(buf))?)
        {
            Ok(()) => Ok(buf.len()),
            Err(fmt::Error) => Err(io::ErrorKind::Other)?,
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
