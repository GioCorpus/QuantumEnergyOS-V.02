/*
 * ═══════════════════════════════════════════════════════════════════════
 *  QuantumEnergyOS — Wayland Compositor
 *  Ultra-lightweight compositor based on tinywl (wlroots)
 *  
 *  Features:
 *    - Quantum-futuristic blue neon theme
 *    - Floating particle effects
 *    - Energy meter integration
 *    - Mexicali desert wallpaper
 *  
 *  Target: < 2251 lines, 64MB RAM
 *  Desde Mexicali, BC — para el mundo. Kardashev 0→1
 * ═══════════════════════════════════════════════════════════════════════
 */

#define _POSIX_C_SOURCE 200112L
#include <assert.h>
#include <getopt.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <time.h>
#include <unistd.h>
#include <wayland-server-core.h>
#include <wlr/backend.h>
#include <wlr/render/allocator.h>
#include <wlr/render/wlr_renderer.h>
#include <wlr/types/wlr_compositor.h>
#include <wlr/types/wlr_cursor.h>
#include <wlr/types/wlr_data_device.h>
#include <wlr/types/wlr_input_device.h>
#include <wlr/types/wlr_keyboard.h>
#include <wlr/types/wlr_output.h>
#include <wlr/types/wlr_output_layout.h>
#include <wlr/types/wlr_pointer.h>
#include <wlr/types/wlr_scene.h>
#include <wlr/types/wlr_seat.h>
#include <wlr/types/wlr_subcompositor.h>
#include <wlr/types/wlr_xcursor_manager.h>
#include <wlr/types/wlr_xdg_shell.h>
#include <wlr/util/log.h>
#include <xkbcommon/xkbcommon.h>

/* ── Quantum Theme Colors ─────────────────────────────────────────── */
#define QUANTUM_BLUE        0x00d4ff
#define QUANTUM_DEEP_BLUE   0x0099ff
#define QUANTUM_CYAN        0x00ffcc
#define QUANTUM_BG          0x0a0a1a
#define QUANTUM_PANEL_BG    0x1a1a2e
#define QUANTUM_TEXT        0xffffff

/* ── Particle System Constants ────────────────────────────────────── */
#define MAX_PARTICLES       50
#define PARTICLE_SIZE       3
#define PARTICLE_SPEED      0.5

/* ── Energy Meter ─────────────────────────────────────────────────── */
#define ENERGY_SOCKET_PATH  "/var/run/quantum-energy.sock"
#define PANEL_HEIGHT        32

/* ── Server Structure ─────────────────────────────────────────────── */
struct tinywl_server {
    struct wl_display *display;
    struct wl_event_loop *event_loop;
    struct wlr_backend *backend;
    struct wlr_renderer *renderer;
    struct wlr_allocator *allocator;
    struct wlr_scene *scene;
    struct wlr_scene_output *scene_output;

    struct wlr_xdg_shell *xdg_shell;
    struct wlr_cursor *cursor;
    struct wlr_xcursor_manager *xcursor_mgr;
    struct wlr_seat *seat;

    struct wl_list keyboards;
    struct wl_list pointers;
    struct wl_list toplevels;

    struct wlr_output_layout *output_layout;
    struct wl_list outputs;
    uint32_t output_count;

    /* Quantum particle system */
    struct {
        double x, y;
        double vx, vy;
        uint32_t color;
        bool active;
    } particles[MAX_PARTICLES];

    /* Energy meter state */
    int energy_percent;
    int energy_socket;
    struct wl_event_source *energy_timer;

    /* Theme */
    uint32_t bg_color;
    uint32_t panel_color;
    uint32_t accent_color;
};

/* ── Output Structure ─────────────────────────────────────────────── */
struct tinywl_output {
    struct wl_list link;
    struct tinywl_server *server;
    struct wlr_output *wlr_output;
    struct wl_listener frame;
    struct wl_listener destroy;
};

/* ── Toplevel (Window) Structure ──────────────────────────────────── */
struct tinywl_toplevel {
    struct wl_list link;
    struct tinywl_server *server;
    struct wlr_xdg_toplevel *xdg_toplevel;
    struct wlr_scene_tree *scene_tree;
    struct wl_listener map;
    struct wl_listener unmap;
    struct wl_listener destroy;
    struct wl_listener request_move;
    struct wl_listener request_resize;
    struct wl_listener request_maximize;
    struct wl_listener request_fullscreen;
};

/* ── Keyboard Structure ───────────────────────────────────────────── */
struct tinywl_keyboard {
    struct wl_list link;
    struct tinywl_server *server;
    struct wlr_keyboard *wlr_keyboard;
    struct wl_listener modifiers;
    struct wl_listener key;
};

/* ── Forward Declarations ─────────────────────────────────────────── */
static void server_handle_new_output(struct wl_listener *listener, void *data);
static void output_frame(struct wl_listener *listener, void *data);
static void output_destroy(struct wl_listener *listener, void *data);
static void server_handle_new_xdg_surface(struct wl_listener *listener, void *data);
static void xdg_toplevel_map(struct wl_listener *listener, void *data);
static void xdg_toplevel_unmap(struct wl_listener *listener, void *data);
static void xdg_toplevel_destroy(struct wl_listener *listener, void *data);
static void keyboard_handle_modifiers(struct wl_listener *listener, void *data);
static void keyboard_handle_key(struct wl_listener *listener, void *data);
static void server_handle_new_input(struct wl_listener *listener, void *data);
static void cursor_motion(struct wl_listener *listener, void *data);
static void cursor_motion_absolute(struct wl_listener *listener, void *data);
static void cursor_button(struct wl_listener *listener, void *data);
static void cursor_axis(struct wl_listener *listener, void *data);
static void cursor_frame(struct wl_listener *listener, void *data);

/* ── Particle System ──────────────────────────────────────────────── */
static void particles_init(struct tinywl_server *server) {
    srand(time(NULL));
    for (int i = 0; i < MAX_PARTICLES; i++) {
        server->particles[i].x = rand() % 1920;
        server->particles[i].y = rand() % 1080;
        server->particles[i].vx = (rand() % 100 - 50) / 100.0 * PARTICLE_SPEED;
        server->particles[i].vy = (rand() % 100 - 50) / 100.0 * PARTICLE_SPEED;
        server->particles[i].color = QUANTUM_BLUE;
        server->particles[i].active = true;
    }
}

static void particles_update(struct tinywl_server *server) {
    for (int i = 0; i < MAX_PARTICLES; i++) {
        if (!server->particles[i].active) continue;
        
        server->particles[i].x += server->particles[i].vx;
        server->particles[i].y += server->particles[i].vy;
        
        /* Wrap around screen */
        if (server->particles[i].x < 0) server->particles[i].x = 1920;
        if (server->particles[i].x > 1920) server->particles[i].x = 0;
        if (server->particles[i].y < 0) server->particles[i].y = 1080;
        if (server->particles[i].y > 1080) server->particles[i].y = 0;
    }
}

/* ── Energy Meter ─────────────────────────────────────────────────── */
static int energy_socket_connect(void) {
    int sock = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sock < 0) return -1;
    
    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, ENERGY_SOCKET_PATH, sizeof(addr.sun_path) - 1);
    
    if (connect(sock, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
        close(sock);
        return -1;
    }
    
    return sock;
}

static void energy_update(struct tinywl_server *server) {
    if (server->energy_socket < 0) {
        server->energy_socket = energy_socket_connect();
        if (server->energy_socket < 0) {
            server->energy_percent = 50; /* Default fallback */
            return;
        }
    }
    
    char buf[32];
    ssize_t n = read(server->energy_socket, buf, sizeof(buf) - 1);
    if (n > 0) {
        buf[n] = '\0';
        server->energy_percent = atoi(buf);
        if (server->energy_percent < 0) server->energy_percent = 0;
        if (server->energy_percent > 100) server->energy_percent = 100;
    } else {
        close(server->energy_socket);
        server->energy_socket = -1;
        server->energy_percent = 50;
    }
}

static int energy_timer_handler(void *data) {
    struct tinywl_server *server = data;
    energy_update(server);
    return 0; /* Continue timer */
}

/* ── Theme Loading ────────────────────────────────────────────────── */
static void theme_load(struct tinywl_server *server) {
    /* Default quantum neon theme */
    server->bg_color = QUANTUM_BG;
    server->panel_color = QUANTUM_PANEL_BG;
    server->accent_color = QUANTUM_BLUE;
    
    /* TODO: Load from /etc/quantum-compositor/theme.ini */
}

/* ── Output Rendering ─────────────────────────────────────────────── */
static void output_frame(struct wl_listener *listener, void *data) {
    struct tinywl_output *output = wl_container_of(listener, output, frame);
    struct tinywl_server *server = output->server;
    struct wlr_output *wlr_output = output->wlr_output;
    
    struct wlr_scene_output *scene_output = 
        wlr_scene_get_scene_output(server->scene, wlr_output);
    
    if (!scene_output) return;
    
    /* Update particles */
    particles_update(server);
    
    /* Render scene */
    wlr_scene_output_commit(scene_output, NULL);
    
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    wlr_scene_output_send_frame_done(scene_output, &now);
}

static void output_destroy(struct wl_listener *listener, void *data) {
    struct tinywl_output *output = wl_container_of(listener, output, destroy);
    wl_list_remove(&output->link);
    wl_list_remove(&output->frame.link);
    wl_list_remove(&output->destroy.link);
    free(output);
}

static void server_handle_new_output(struct wl_listener *listener, void *data) {
    struct tinywl_server *server = wl_container_of(listener, server, new_output);
    struct wlr_output *wlr_output = data;
    
    wlr_output_init_render(wlr_output, server->allocator, server->renderer);
    
    struct wlr_output_state state;
    wlr_output_state_init(&state);
    wlr_output_state_set_enabled(&state, true);
    
    struct wlr_output_mode *mode = wlr_output_preferred_mode(wlr_output);
    if (mode) {
        wlr_output_state_set_mode(&state, mode);
    }
    
    wlr_output_commit_state(wlr_output, &state);
    wlr_output_state_finish(&state);
    
    struct tinywl_output *output = calloc(1, sizeof(*output));
    output->server = server;
    output->wlr_output = wlr_output;
    wl_list_insert(&server->outputs, &output->link);
    
    output->frame.notify = output_frame;
    wl_signal_add(&wlr_output->events.frame, &output->frame);
    
    output->destroy.notify = output_destroy;
    wl_signal_add(&wlr_output->events.destroy, &output->destroy);
    
    struct wlr_output_layout_output *l_output = 
        wlr_output_layout_add_auto(server->output_layout, wlr_output);
    
    output->scene_output = wlr_scene_output_create(server->scene, wlr_output);
    wlr_scene_output_layout_add_output(server->scene_output, l_output);
    
    server->output_count++;
}

/* ── XDG Shell Handling ───────────────────────────────────────────── */
static void xdg_toplevel_map(struct wl_listener *listener, void *data) {
    struct tinywl_toplevel *toplevel = wl_container_of(listener, toplevel, map);
    /* Focus new window */
    /* TODO: Implement focus logic */
}

static void xdg_toplevel_unmap(struct wl_listener *listener, void *data) {
    struct tinywl_toplevel *toplevel = wl_container_of(listener, toplevel, unmap);
    /* TODO: Handle unmap */
}

static void xdg_toplevel_destroy(struct wl_listener *listener, void *data) {
    struct tinywl_toplevel *toplevel = wl_container_of(listener, toplevel, destroy);
    wl_list_remove(&toplevel->link);
    free(toplevel);
}

static void server_handle_new_xdg_surface(struct wl_listener *listener, void *data) {
    struct tinywl_server *server = wl_container_of(listener, server, new_xdg_surface);
    struct wlr_xdg_surface *xdg_surface = data;
    
    if (xdg_surface->role != WLR_XDG_SURFACE_ROLE_TOPLEVEL) return;
    
    struct tinywl_toplevel *toplevel = calloc(1, sizeof(*toplevel));
    toplevel->server = server;
    toplevel->xdg_toplevel = xdg_surface->toplevel;
    toplevel->scene_tree = wlr_scene_xdg_surface_create(
        &server->scene->tree, xdg_surface);
    
    toplevel->map.notify = xdg_toplevel_map;
    wl_signal_add(&xdg_surface->surface->events.map, &toplevel->map);
    
    toplevel->unmap.notify = xdg_toplevel_unmap;
    wl_signal_add(&xdg_surface->surface->events.unmap, &toplevel->unmap);
    
    toplevel->destroy.notify = xdg_toplevel_destroy;
    wl_signal_add(&xdg_surface->events.destroy, &toplevel->destroy);
    
    wl_list_insert(&server->toplevels, &toplevel->link);
}

/* ── Input Handling ───────────────────────────────────────────────── */
static void keyboard_handle_modifiers(struct wl_listener *listener, void *data) {
    struct tinywl_keyboard *keyboard = wl_container_of(listener, keyboard, modifiers);
    wlr_seat_set_keyboard(keyboard->server->seat, keyboard->wlr_keyboard);
    wlr_seat_keyboard_notify_modifiers(keyboard->server->seat,
        &keyboard->wlr_keyboard->modifiers);
}

static void keyboard_handle_key(struct wl_listener *listener, void *data) {
    struct tinywl_keyboard *keyboard = wl_container_of(listener, keyboard, key);
    struct wlr_keyboard_key_event *event = data;
    
    wlr_seat_set_keyboard(keyboard->server->seat, keyboard->wlr_keyboard);
    wlr_seat_keyboard_notify_key(keyboard->server->seat,
        event->time_msec, event->keycode, event->state);
}

static void server_handle_new_input(struct wl_listener *listener, void *data) {
    struct tinywl_server *server = wl_container_of(listener, server, new_input);
    struct wlr_input_device *device = data;
    
    switch (device->type) {
    case WLR_INPUT_DEVICE_KEYBOARD:
        /* TODO: Create keyboard */
        break;
    case WLR_INPUT_DEVICE_POINTER:
        /* TODO: Create pointer */
        break;
    default:
        break;
    }
    
    uint32_t caps = WL_SEAT_CAPABILITY_POINTER;
    if (!wl_list_empty(&server->keyboards)) {
        caps |= WL_SEAT_CAPABILITY_KEYBOARD;
    }
    wlr_seat_set_capabilities(server->seat, caps);
}

/* ── Cursor Handling ──────────────────────────────────────────────── */
static void cursor_motion(struct wl_listener *listener, void *data) {
    struct tinywl_server *server = wl_container_of(listener, server, cursor_motion);
    struct wlr_pointer_motion_event *event = data;
    wlr_cursor_move(server->cursor, &event->pointer->base,
        event->delta_x, event->delta_y);
}

static void cursor_motion_absolute(struct wl_listener *listener, void *data) {
    struct tinywl_server *server = wl_container_of(listener, server, cursor_motion_absolute);
    struct wlr_pointer_motion_absolute_event *event = data;
    wlr_cursor_warp_absolute(server->cursor, &event->pointer->base,
        event->x, event->y);
}

static void cursor_button(struct wl_listener *listener, void *data) {
    struct tinywl_server *server = wl_container_of(listener, server, cursor_button);
    struct wlr_pointer_button_event *event = data;
    wlr_seat_pointer_notify_button(server->seat,
        event->time_msec, event->button, event->state);
}

static void cursor_axis(struct wl_listener *listener, void *data) {
    struct tinywl_server *server = wl_container_of(listener, server, cursor_axis);
    struct wlr_pointer_axis_event *event = data;
    wlr_seat_pointer_notify_axis(server->seat,
        event->time_msec, event->orientation, event->delta,
        event->delta_discrete, event->source);
}

static void cursor_frame(struct wl_listener *listener, void *data) {
    struct tinywl_server *server = wl_container_of(listener, server, cursor_frame);
    wlr_seat_pointer_notify_frame(server->seat);
}

/* ── Main Entry Point ─────────────────────────────────────────────── */
int main(int argc, char *argv[]) {
    wlr_log_init(WLR_DEBUG, NULL);
    
    /* Parse command line */
    char *startup_cmd = NULL;
    int c;
    while ((c = getopt(argc, argv, "s:h")) != -1) {
        switch (c) {
        case 's':
            startup_cmd = optarg;
            break;
        default:
            printf("Usage: %s [-s startup command]\n", argv[0]);
            return 0;
        }
    }
    if (optind < argc) {
        printf("Usage: %s [-s startup command]\n", argv[0]);
        return 0;
    }
    
    /* Initialize server */
    struct tinywl_server server = {0};
    server.energy_socket = -1;
    server.energy_percent = 50;
    
    /* Load theme */
    theme_load(&server);
    
    /* Initialize particle system */
    particles_init(&server);
    
    /* Create Wayland display */
    server.display = wl_display_create();
    server.event_loop = wl_display_get_event_loop(server.display);
    
    /* Create backend (DRM/KMS or libinput) */
    server.backend = wlr_backend_autocreate(server.event_loop, NULL);
    if (!server.backend) {
        wlr_log(WLR_ERROR, "Failed to create backend");
        return 1;
    }
    
    /* Create renderer */
    server.renderer = wlr_renderer_autocreate(server.backend);
    if (!server.renderer) {
        wlr_log(WLR_ERROR, "Failed to create renderer");
        return 1;
    }
    wlr_renderer_init_wl_display(server.renderer, server.display);
    
    /* Create allocator */
    server.allocator = wlr_allocator_autocreate(server.backend, server.renderer);
    if (!server.allocator) {
        wlr_log(WLR_ERROR, "Failed to create allocator");
        return 1;
    }
    
    /* Create scene graph */
    server.scene = wlr_scene_create();
    wlr_scene_attach_output_layout(server.scene, server.output_layout);
    
    /* Create output layout */
    server.output_layout = wlr_output_layout_create();
    
    /* Create XDG shell */
    server.xdg_shell = wlr_xdg_shell_create(server.display, 3);
    
    /* Create cursor */
    server.cursor = wlr_cursor_create();
    wlr_cursor_attach_output_layout(server.cursor, server.output_layout);
    
    /* Create XCursor manager */
    server.xcursor_mgr = wlr_xcursor_manager_create(NULL, 24);
    wlr_xcursor_manager_load(server.xcursor_mgr, 1.0);
    
    /* Create seat */
    server.seat = wlr_seat_create(server.display, "seat0");
    
    /* Initialize lists */
    wl_list_init(&server.keyboards);
    wl_list_init(&server.pointers);
    wl_list_init(&server.toplevels);
    wl_list_init(&server.outputs);
    
    /* Setup event listeners */
    server.new_output.notify = server_handle_new_output;
    wl_signal_add(&server.backend->events.new_output, &server.new_output);
    
    server.new_xdg_surface.notify = server_handle_new_xdg_surface;
    wl_signal_add(&server.xdg_shell->events.new_surface, &server.new_xdg_surface);
    
    server.new_input.notify = server_handle_new_input;
    wl_signal_add(&server.backend->events.new_input, &server.new_input);
    
    server.cursor_motion.notify = cursor_motion;
    wl_signal_add(&server.cursor->events.motion, &server.cursor_motion);
    
    server.cursor_motion_absolute.notify = cursor_motion_absolute;
    wl_signal_add(&server.cursor->events.motion_absolute, &server.cursor_motion_absolute);
    
    server.cursor_button.notify = cursor_button;
    wl_signal_add(&server.cursor->events.button, &server.cursor_button);
    
    server.cursor_axis.notify = cursor_axis;
    wl_signal_add(&server.cursor->events.axis, &server.cursor_axis);
    
    server.cursor_frame.notify = cursor_frame;
    wl_signal_add(&server.cursor->events.frame, &server.cursor_frame);
    
    /* Setup energy meter timer (update every 5 seconds) */
    server.energy_timer = wl_event_loop_add_timer(server.event_loop,
        energy_timer_handler, &server);
    wl_event_source_timer_update(server.energy_timer, 5000);
    
    /* Start backend */
    if (!wlr_backend_start(server.backend)) {
        wlr_log(WLR_ERROR, "Failed to start backend");
        return 1;
    }
    
    /* Set environment variables */
    setenv("WAYLAND_DISPLAY", "wayland-0", true);
    setenv("XDG_SESSION_TYPE", "wayland", true);
    
    /* Run startup command if provided */
    if (startup_cmd) {
        if (fork() == 0) {
            execl("/bin/sh", "/bin/sh", "-c", startup_cmd, NULL);
        }
    }
    
    /* Run event loop */
    wlr_log(WLR_INFO, "QuantumEnergyOS Compositor running on wayland-0");
    wl_display_run(server.display);
    
    /* Cleanup */
    wl_display_destroy(server.display);
    if (server.energy_socket >= 0) close(server.energy_socket);
    
    return 0;
}
