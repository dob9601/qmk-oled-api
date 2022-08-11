[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/A0A1EB7YD)

# ![QMK Oled API](https://user-images.githubusercontent.com/24723950/183790643-aa12642c-0f83-4545-844e-621f3601720d.png)

Drawing graphics on a QMK Oled screen is harder than it should be! This project hopes to resolve that.

This crate provides an API for drawing to the OLED screen on your QMK keyboard, along with a small snippet required to turn your keyboard into a client for it.

## Showcase

Below are some projects that have been built on top of this API:

- qmk-nowplaying [TBC]

## Client Snippet

Below is a snippet of config you can use to turn your keyboard into a client:
```c
#include "raw_hid.h"
#include "print.h"

#include "string.h"

// What the screen look like if no connection is established. This shows a "No connection" message
char current_screen[] = {
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, 56, 68,  4,  4,  4,  4,  4, 68, 56,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,128,128,  0, 48,248,240,224,128,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, 78, 81, 81, 81, 81, 81,206,  0,  0,  0,  0,  0,  0,  0,  0,136,200,200,136,168,168,152,152,136,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,129,195,  6, 12,156, 56,112,195,135,255,254,252,192,  3, 31,255,254,240,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, 20, 20, 20, 20, 20, 20,243,  0,  0,  0,  0,  0,  0,  0,  0, 28, 34, 34, 34, 34, 34, 28,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  1,  3,  0,  8, 31, 31, 14,192,241,243,103, 15, 31, 48,120,255,255, 15,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, 
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, 25, 25,  1,  1,  1,  1,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  1,  1,  0, 14, 31, 15,  6,  0,  1,  3,  6,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
};

/*
 * Payload structure. Index identifies where on the OLED to write to. 
 * Report IDs aren't used but cause a pain (not writing, occasionally being stripped off etc.)
 * For this reason, the first byte should always be "1"
 * |  1  | 2 | 3 --------- 32 |
 * |REPID|IDX|     DATA       |
 */
static const int PAYLOAD_SIZE = 32;

void raw_hid_receive(uint8_t *data, uint8_t length) {
    // TODO: Read report ID to determine the OLED screen to write to
    raw_hid_send(data, length);
    uint8_t* index = &data[1];

    memcpy(&current_screen[(PAYLOAD_SIZE - 2) * (*index)], &data[2], (PAYLOAD_SIZE - 2));
}


static void render_oled(void) {
    oled_write_raw(current_screen, sizeof(current_screen));
}

bool oled_task_user(void) {
    render_oled();
    return false;
}
```


## Roadmap

| Feature               | Implemented |
| ---------------       | :---------: |
| Image rendering       |     ✅      |
| Basic shape rendering |     ✅      |
| Text rendering        |     ✅      |

