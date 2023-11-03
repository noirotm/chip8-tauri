const { invoke } = window.__TAURI__.tauri;
const { appWindow } = window.__TAURI__.window;

const SCREEN_WIDTH = 64;
const SCREEN_HEIGHT = 32;

let screen;
let ctx;

let settings = {
    rendering: "pixelated",
    backgroundColor: "rgb(0 0 0)",
    color: "rgb(0 128 0)"
};

window.addEventListener("DOMContentLoaded", () => {
    screen = document.querySelector("#screen");
    screen.width = SCREEN_WIDTH;
    screen.height = SCREEN_HEIGHT;

    ctx = screen.getContext("2d");
});

window.addEventListener("keydown", (ev) => {
    if (!ev.repeat) {
        invoke("key_down", {key: ev.key});
    }
});

window.addEventListener("keyup", (ev) => {
    invoke("key_up", { key: ev.key });
});

appWindow.listen('clear', () => {
    ctx.fillStyle = "rgb(0, 0, 0)";
    ctx.fillRect(0, 0, screen.width, screen.height);
});

appWindow.listen('draw', ({_, payload}) => {
    for (let y = 0; y < SCREEN_HEIGHT; y++) {
        for (let x = 0; x < SCREEN_WIDTH; x++) {
            const i = SCREEN_WIDTH * y + x;

            if (payload.pixels[i] === true) {
                ctx.fillStyle = settings.color;
            } else {
                ctx.fillStyle = settings.backgroundColor;
            }

            ctx.fillRect(x, y, 1, 1);
        }
    }
});
