in vec2 v_uv;
out vec4 frag;

uniform usampler2D tex;

void main() {
    vec4 color = texture(tex, v_uv);
    frag = color/255;
}