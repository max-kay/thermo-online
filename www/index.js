import { SmallModel, MediumModel, BigModel, XBigModel, get_color, make_energies } from "thermo-online";
import { memory } from "thermo-online/thermo_online_bg";
import Plotly from 'plotly.js-dist-min';

let tempSteps = 10;
let startTemp = 8.0;
let eSteps = 100;
let mSteps = 100;
let nFrames = 3;

const energies = make_energies(-0.75, -0.25, -0.75);

let Model;
let model;

const COLOR_1 = get_color(0);
const COLOR_2 = get_color(1);
const COLOR_3 = get_color(2);

window.addEventListener("beforeunload", function () { model = none })

function runSimulation() {
    model = undefined;
    document.getElementById("modelOutput").style.display = "none";

    document.getElementById("run").disabled = true;

    let method = document.getElementById('method').value;
    tempSteps = parseInt(document.getElementById('tempSteps').value);
    startTemp = parseFloat(document.getElementById('startTemp').value);
    eSteps = parseInt(document.getElementById('eSteps').value);
    mSteps = parseInt(document.getElementById('mSteps').value);
    // TODO figure out nFrame in code


    switch (parseInt(document.getElementById('modelSize').value)) {
        case 32:
            Model = SmallModel;
            break;
        case 64:
            Model = MediumModel;
            break;
        case 128:
            Model = BigModel;
            break;
        case 256:
            Model = XBigModel;
            break;
        default:
            console.log("failed to assign model");
            console.log(modelSize);
    }

    model = Model.new(energies, 1.0, 1.0, method, tempSteps * nFrames, tempSteps);



    let progressBar = document.getElementById("progress").style;
    console.log(progressBar);
    progressBar.height = "30px";

    for (let i = 0; i < tempSteps; i++) {
        const temp = startTemp * ((tempSteps - 1 - i) / (tempSteps - 1));
        model.run_at_temp(eSteps, mSteps, temp, nFrames);
        console.log(temp);
        console.log(i);
        progressBar.width = i / (tempSteps - 1) * 100 + "%";
        progressBar.innerHTML = (i + 1) + " / " + tempSteps;;
    }


    const gifLen = model.gif_len();
    const gifPtr = model.gif_ptr();

    const gifData = new Uint8Array(memory.buffer, gifPtr, gifLen);
    let blob = new Blob([gifData], { type: 'image/gif' });
    let url = URL.createObjectURL(blob);

    document.getElementById("run").disabled = false;
    document.getElementById("modelOutput").style.display = "block";

    let img = document.getElementById("animation");
    img.src = url;

    const temp = new Float32Array(memory.buffer, model.temp_ptr(), model.log_len());
    const energy = new Float32Array(memory.buffer, model.int_energy_ptr(), model.log_len());
    const heat_capacity = new Float32Array(memory.buffer, model.heat_capacity_ptr(), model.log_len());

    // plot temp energy
    const trace1 = {
        x: temp,
        y: energy,
    };
    const layout1 = {
        xaxis: {
            title: 'Temperature',
        },
        yaxis: {
            title: 'Energy',
        },
    };
    Plotly.newPlot('energyTemp', [trace1], layout1);

    // plot temp energy
    const trace2 = {
        x: temp,
        y: heat_capacity,
    };
    const layout2 = {
        xaxis: {
            title: 'Temperature',
        },
        yaxis: {
            title: 'Heat Capacity',
        },
    };
    Plotly.newPlot('capacityTemp', [trace2], layout2);

}

window.runSimulation = runSimulation;