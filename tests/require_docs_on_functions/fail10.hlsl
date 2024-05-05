/**
* Function docs (missing variable with HLSL semantic).
* 
* @param iThreadIdInGroup Int docs.
*/
void foo(
    uint3 threadIdInDispatch : SV_DispatchThreadID,
    uint iThreadIdInGroup : SV_GroupIndex) {
    int a = 2;
}