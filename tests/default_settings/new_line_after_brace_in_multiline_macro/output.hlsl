#define MULTI_LINE_MACRO  \
if (a > 2) {\         
    b = 2;                \
}else if (a < 2) {\  
    b = 1;                \
}                         \   
else {\
    b = 0;                \
}

int a = 3;
MULTI_LINE_MACRO