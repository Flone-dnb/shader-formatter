#somemacro {
#if defined(FEATURE1) || defined(FEATURE2) // `defined(...)` should not be parsed as a function
    uint iMyVariable;
#endif
}