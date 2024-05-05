/**
* Function docs (semantic docs not required).
*
* @param test1 Docs.
* @param test2 Docs.
*/
void foo(
    float test1,
    uint3 threadIdInDispatch : SV_DispatchThreadID,
    uint iThreadIdInGroup : SV_GroupIndex,
    uint test2) {
    int a = 2;
}