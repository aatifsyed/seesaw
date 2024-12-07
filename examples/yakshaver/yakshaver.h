typedef struct yakshaver yakshaver;

/** create a yakshaver */
yakshaver *create(void);
/** destroy a yakshaver */
void destroy(yakshaver *);
/** get number of yaks shaved */
unsigned int yaks_shaved(const yakshaver *);
/** shave some yaks */
int shave(yakshaver *);
