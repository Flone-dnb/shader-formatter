struct Foo {
    int iValue = 2;
};

int g_iGlobalVar = 3; {
    int g_iGlobalNested = 0;
}

int g_iGlobalShift<some_meta> = 0;

void foo(int iValue) {
    int iTest = 0;
}

void test() {
    int iLocalVar = 2; {
        int iNested = 3;
    }
}

int g_iAnotherGlobal = 1;