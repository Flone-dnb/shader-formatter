#CUSTOM_MACRO1 layout(binding = 0) uniform ComputeInfo {
    #CUSTOM_MACRO2 struct ComputeInfo {
        uint iThreadGroupCountX;
        #CUSTOM_MACRO3 } computeInfo;
    #CUSTOM_MACRO4 }; ConstantBuffer<ComputeInfo> computeInfo : register(b0, space5);