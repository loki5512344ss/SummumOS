/* SPDX-License-Identifier: GPL-2.0-or-later */
#define main tapok_engine_main
#define DG_DrawFrame tapok_draw_frame
#include "tapok_sdl.c"
#undef DG_DrawFrame
#undef main
#include <assert.h>
#include <sys/stat.h>
#include "doomstat.h"
#include "p_local.h"
#include "g_game.h"
#include "p_saveg.h"
#include "r_state.h"
#include "s_sound.h"
#include "sounds.h"
#ifdef FEATURE_SOUND
#include <SDL_mixer.h>
static SDL_atomic_t samples;
static void proverit_zvuk(void *context, Uint8 *stream, int len)
{
    int i;
    (void)context;
    for (i = 0; i < len; ++i) if (stream[i]) { SDL_AtomicAdd(&samples, 1); break; }
}
#endif

void P_FireWeapon(player_t *player);
extern gameaction_t gameaction;

static int saved_health;
static fixed_t saved_x;
static sector_t *door_sector;
static fixed_t door_height;
static int fired;
static int stage;

void DG_DrawFrame(void)
{
    struct stat st;
    int i;
    tapok_draw_frame();
    assert(SDL_GetTicks() < 15000);
    if (stage == 0 && leveltime >= 5) {
        assert(gamestate == GS_LEVEL);
        assert(numsectors > 0 && numnodes > 0 && numlines > 0);
        assert(players[0].mo != NULL);
        for (i = 0; i < numlines; ++i) {
            if ((lines[i].special == 1 || lines[i].special == 31 || lines[i].special == 117 || lines[i].special == 118) && lines[i].backsector) {
                door_sector = lines[i].backsector;
                door_height = door_sector->ceilingheight;
                EV_VerticalDoor(&lines[i], players[0].mo);
                if (door_sector->specialdata) break;
                door_sector = NULL;
            }
        }
        assert(door_sector != NULL);
        stage = 1;
    }
    else if (stage == 1 && leveltime >= 10) {
        assert(door_sector->ceilingheight != door_height);
        players[0].readyweapon = wp_pistol;
        players[0].pendingweapon = wp_nochange;
        players[0].ammo[am_clip] = 50;
#ifdef FEATURE_SOUND
        if (getenv("TAPOK_DOOM_VERIFY_AUDIO")) {
            int rate, channels;
            Uint16 format;
            assert(Mix_QuerySpec(&rate, &format, &channels));
            Mix_SetPostMix(proverit_zvuk, NULL);
            S_StartSound(NULL, sfx_pistol);
        }
#endif
        P_FireWeapon(&players[0]);
        stage = 2;
    }
    else if (stage == 2 && leveltime >= 20) {
        assert(players[0].ammo[am_clip] < 50);
#ifdef FEATURE_SOUND
        if (getenv("TAPOK_DOOM_VERIFY_AUDIO")) {
            assert(SDL_AtomicGet(&samples) > 0);
            printf("Doom audio: nonzero mixed buffers=%d, music_playing=%d\n", SDL_AtomicGet(&samples), Mix_PlayingMusic());
        }
#endif
        {
            int kills = players[0].killcount;
            mobj_t *enemy = P_SpawnMobj(players[0].mo->x + 64 * FRACUNIT,
                players[0].mo->y, ONFLOORZ, MT_POSSESSED);
            assert(enemy && enemy->health > 0);
            P_DamageMobj(enemy, players[0].mo, players[0].mo, 1000);
            assert(enemy->health <= 0 && (enemy->flags & MF_CORPSE));
            assert(players[0].killcount == kills + 1);
        }
        fired = 1;
        saved_health = players[0].health;
        saved_x = players[0].mo->x;
        G_SaveGame(0, "TAPOK ROUNDTRIP");
        stage = 3;
    }
    else if (stage == 3 && leveltime >= 40) {
        char *path = P_SaveGameFile(0);
        assert(stat(path, &st) == 0 && st.st_size > 100);
        players[0].health = players[0].mo->health = 13;
        players[0].mo->x += FRACUNIT;
        G_LoadGame(path);
        stage = 4;
    }
    else if (stage == 4 && gameaction == ga_nothing && leveltime < 40) {
        assert(fired);
        assert(players[0].health == saved_health);
        assert(players[0].mo->x == saved_x);
        assert(gamestate == GS_LEVEL);
        puts("Doom gameplay tests passed: BSP level, door, pistol, monster damage/kill, save/load state");
        exit(0);
    }
}

int main(int argc, char **argv)
{
    assert(!getenv("TAPOK_DOOM_FRAMES"));
    doomgeneric_Create(argc, argv);
    for (;;) doomgeneric_Tick();
}
