SHADERS = \
	  vitral/src/sprite.frag.spv \
	  vitral/src/sprite.vert.spv \
	  vitral/src/blit.frag.spv \
	  vitral/src/blit.vert.spv \

all:
	cargo build --release

shaders: $(SHADERS)

%.spv: %
	glslc $< -o $@
