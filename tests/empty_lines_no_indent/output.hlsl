uniform vec4 color;

void main() {
    background = vec4(0.2, 0.3, 0.4, 1.0);

    color = background;

    #ifdef FOO
        int a = 2;

        int b = 2;

        #ifdef BAR
            a = 3;

            b = 3;
        #endif
    #endif
}