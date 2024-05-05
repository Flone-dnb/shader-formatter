void foo(
    uint3 threadIdInDispatch : SV_DispatchThreadID,
    uint iThreadIdInGroup : SV_GroupIndex) {
    int a = 2;
}