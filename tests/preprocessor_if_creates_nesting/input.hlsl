#define FOO

void bar() {
    #define SOMEVAL1
    int test = 0;

#if FEATURE
// hmm...
#endif
    
        #ifdef ENABLE_FEATURE1
    int a = 1;

    #ifdef SUB_FEATURE
    #define SOME_MACRO
    #endif
#elif ENABLE_FEATURE2
    int a = 2;
#else
    int a = 3;
#endif
    int b = 0;
    
    // #ifdef DISABLED
    // int a = 2;
    // #endif
}