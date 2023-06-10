import { TinyModel, SmallModel, MediumModel } from "thermo-online";
import { memory } from "thermo-online/thermo_online_bg";

const CELL_SIZE = 5;
const TEMP_STEPS = 50;
const START_TEMP = 1.0;
const E_STEPS = 100;
const M_STEPS = 100;
const N_FRAMES = 3;

const model = TinyModel.new([-1.0, 1.0, 1.0, 1.0], 1.0, 1.0, "move_vacancy", TEMP_STEPS * N_FRAMES, TEMP_STEPS);
const width = model.width();
const height = model.height();

const canvas = document.getElementById("main_animation");
canvas.height = CELL_SIZE * height;
canvas.width = CELL_SIZE * width;
const ctx = canvas.getContext('2d');


const getIndex = (row, column) => {
    return row * width + column;
};

const drawGrid = (array) => {
    ctx.beginPath();

    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
            const idx = getIndex(row, col)
            ctx.fillStyle = array[idx] === 0 ? "orange" : "red"

            ctx.fillRect(
                col * CELL_SIZE,
                row * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE,
            );
        }
    }
    ctx.stroke()
}

var temp_step = TEMP_STEPS;
const coolingLoop = () => {
    temp_step -= 1
    const temp = START_TEMP * (temp_step / (TEMP_STEPS - 1))
    model.run_at_temp(E_STEPS, M_STEPS, temp, 1)
    const frame = new Uint8Array(memory.buffer, model.anim_start_ptr(), width * height);
    console.log(temp)
    console.log(temp_step)
    drawGrid(frame)
    if (temp_step === 0) {
        return;
    }
    requestAnimationFrame(coolingLoop)
}

var currentFrame = 0;
const renderLoop = () => {
    {
        let animation = new Uint8Array(memory.buffer, model.anim_frame_ptr(currentFrame), width * height);
        drawGrid(animation)
        currentFrame = (currentFrame + 1) % animationLen
    }
    requestAnimationFrame(renderLoop);
}


coolingLoop()
const animationLen = model.anim_len();

renderLoop()