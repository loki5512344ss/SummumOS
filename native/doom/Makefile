# Тапок ел компилятор. Компилятор согласился быть супом.
CC ?= cc
PKG_CONFIG ?= pkg-config
OUT ?= ../../target/doom
CFLAGS ?= -O2 -g
CPPFLAGS += -D_DEFAULT_SOURCE -DNORMALUNIX -DLINUX
SDL_FLAGS := $(shell $(PKG_CONFIG) --cflags sdl2)
SDL_LIBS := $(shell $(PKG_CONFIG) --libs sdl2)
MIXER := $(shell $(PKG_CONFIG) --exists SDL2_mixer && echo yes)
SOURCES = dummy.c am_map.c doomdef.c doomstat.c dstrings.c d_event.c d_items.c d_iwad.c d_loop.c d_main.c d_mode.c d_net.c f_finale.c f_wipe.c g_game.c hu_lib.c hu_stuff.c info.c i_cdmus.c i_endoom.c i_joystick.c i_scale.c i_sound.c i_system.c i_timer.c memio.c m_argv.c m_bbox.c m_cheat.c m_config.c m_controls.c m_fixed.c m_menu.c m_misc.c m_random.c p_ceilng.c p_doors.c p_enemy.c p_floor.c p_inter.c p_lights.c p_map.c p_maputl.c p_mobj.c p_plats.c p_pspr.c p_saveg.c p_setup.c p_sight.c p_spec.c p_switch.c p_telept.c p_tick.c p_user.c r_bsp.c r_data.c r_draw.c r_main.c r_plane.c r_segs.c r_sky.c r_things.c sha1.c sounds.c statdump.c st_lib.c st_stuff.c s_sound.c tables.c v_video.c wi_stuff.c w_checksum.c w_file.c w_main.c w_wad.c z_zone.c w_file_stdc.c i_input.c i_video.c doomgeneric.c tapok_sdl.c
ifeq ($(MIXER),yes)
CPPFLAGS += -DFEATURE_SOUND $(shell $(PKG_CONFIG) --cflags SDL2_mixer)
SDL_LIBS += $(shell $(PKG_CONFIG) --libs SDL2_mixer)
SOURCES += mus2mid.c i_sdlmusic.c i_sdlsound.c
MODE = sound
else
MODE = silent
endif
OBJECTS = $(addprefix $(OUT)/$(MODE)/,$(SOURCES:.c=.o))
HEADERS = $(wildcard core/*.h)
.PHONY: all
all: $(OUT)/tapok-doom
$(OUT)/tapok-doom: $(OBJECTS) FORCE
	$(CC) $(LDFLAGS) $(OBJECTS) -o $@ $(SDL_LIBS) -lm
	@echo "Doom backend: $(MODE), $@"
.PHONY: FORCE
FORCE:
$(OUT)/$(MODE)/%.o: core/%.c $(HEADERS)
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) $(SDL_FLAGS) -c $< -o $@
$(OUT)/$(MODE)/tapok_sdl.o: tapok_sdl.c $(HEADERS)
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) $(SDL_FLAGS) -Icore -c $< -o $@

.PHONY: test
TEST_OBJECTS = $(filter-out $(OUT)/$(MODE)/tapok_sdl.o,$(OBJECTS))
test: $(OUT)/test-backend
	$(OUT)/test-backend
$(OUT)/test-backend: test_backend.c tapok_sdl.c $(TEST_OBJECTS) $(HEADERS) FORCE
	$(CC) $(CPPFLAGS) $(CFLAGS) $(SDL_FLAGS) -Icore test_backend.c $(TEST_OBJECTS) -o $@ $(SDL_LIBS) -lm

$(OUT)/test-gameplay: test_gameplay.c tapok_sdl.c $(TEST_OBJECTS) $(HEADERS) FORCE
	$(CC) $(CPPFLAGS) $(CFLAGS) $(SDL_FLAGS) -Icore test_gameplay.c $(TEST_OBJECTS) -o $@ $(SDL_LIBS) -lm
SOUND_TEST_FLAGS ?= -nosound
.PHONY: gameplay-test
gameplay-test: $(OUT)/test-gameplay
	@test -n "$(IWAD)" || (echo 'Supply IWAD=/path/to/freedoom2.wad' >&2; exit 2)
	@mkdir -p $(OUT)/qa-saves
	SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy $(OUT)/test-gameplay -iwad "$(IWAD)" -warp 1 -skill 3 -nomonsters $(SOUND_TEST_FLAGS) -savedir $(OUT)/qa-saves -config $(OUT)/qa.cfg -extraconfig $(OUT)/qa-extra.cfg
