/**
* Function docs.
*/
void foo(
    uint3 threadIdInDispatch : SV_DispatchThreadID,
    uint iThreadIdInGroup : SV_GroupIndex,
    float test) {
    int a = 2;
}