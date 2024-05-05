// foo(
// 1,
// 2);

void foo() {
    int array1 = [
        1,
        2,
        3];
    
    foo(1, 2);
    
    foo(
        1,
        2);
    
    foo(
        1,
        2,
        bar1(3, 4),
        bar2(
            5,
            6));
    
    int array2 = [
        1,   2,   3
        4,   5,   6
        7,   8,   9];
    
    int array3 = [
        [1, 2, 3],
        [1, 2, 3]];
}