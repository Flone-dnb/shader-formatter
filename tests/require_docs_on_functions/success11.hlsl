/**
 * This function is not expected be called from other shader files.
 *
 * Testing that `return foo(...)` does not require documentation.
 *
 * @param vertexIn Input vertex.
 *
 * @return Vertex.
 */
VertexOut main(VertexIn vertexIn) {
    return foo(vertexIn);
}
