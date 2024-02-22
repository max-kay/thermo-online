import { XSmallModel, SmallModel, MediumModel, BigModel, XBigModel, get_color, make_energies } from "thermo-online";
import { memory } from "thermo-online/thermo_online_bg";
import { gsap } from "gsap";
import Plotly from "plotly.js-dist-min";


const GIF_DURATION = 10.0;

let distrPerTemp = 1000;
let tempSteps;
let startTemp;
let endTemp
let eSteps;
let mSteps;
let nFrames;
let cA;
let cB;
let method;
let energies;

let ModelType;
let modelInstance;


// update colors for legend
document.styleSheets[0].insertRule(".color0{ background:" + get_color(0) + ";}")
document.styleSheets[0].insertRule(".color1{ background:" + get_color(1) + ";}")

function readInputs() {
    method = document.getElementById('method').value;
    tempSteps = parseInt(document.getElementById('tempSteps').value);
    startTemp = parseFloat(document.getElementById('startTemp').value);
    endTemp = parseFloat(document.getElementById('endTemp').value);
    eSteps = parseInt(document.getElementById('eSteps').value);
    mSteps = parseInt(document.getElementById('mSteps').value);

    energies = make_energies(
        parseFloat(document.getElementById("j00").value),
        parseFloat(document.getElementById("j01").value),
        parseFloat(document.getElementById("j11").value),
    )
    cA = parseFloat(document.getElementById("cA").value);
    cB = parseFloat(document.getElementById("cB").value);
    switch (parseInt(document.getElementById('modelSize').value)) {
        case 16:
            ModelType = XSmallModel;
            break;
        case 32:
            ModelType = SmallModel;
            break;
        case 64:
            ModelType = MediumModel;
            break;
        case 128:
            ModelType = BigModel;
            break;
        case 256:
            ModelType = XBigModel;
            break;
        default:
            console.log("failed to assign model, using SmallModel");
            console.log("modelSize: " + modelSize);
            ModelType = SmallModel
    }
}

function setUiRunning() {
    gsap.to("#run", { disabled: true, duration: 0 })
    gsap.to("#modelOutput", { display: "none", duration: 0 })
    gsap.to("#running", { display: "block", duration: 0 })
    gsap.to("#progress", { width: "0%", duration: 0 })
}
function setUiOutput() {
    gsap.to("#run", { disabled: false, innerHTML: "Rerun", duration: 0.01 })
    gsap.to("#modelOutput", { display: "block", duration: 0.01 })
    gsap.to("#running", { display: "none", duration: 0.02 })

}

function runSimulation() {
    modelInstance = undefined;
    readInputs();
    setUiRunning();
    nFrames = Math.max(Math.round(GIF_DURATION / tempSteps / 0.1), 1); // for around 10fps
    let sPerFrame = GIF_DURATION / tempSteps / nFrames;
    modelInstance = ModelType.new(energies, cA, cB, method, tempSteps, Math.round(sPerFrame * 100), distrPerTemp);
    let a = (Math.log(endTemp) - Math.log(startTemp)) / (tempSteps - 1);
    function animateSimulation(i) {
        if (i < tempSteps) {
            const temp = startTemp * Math.exp(a * i);
            modelInstance.run_at_temp(eSteps, mSteps, temp, nFrames, distrPerTemp);
            gsap.to("#progress", { duration: 0, width: (i / (tempSteps - 1)) * 100 + "%" });
            i++;
            requestAnimationFrame(() => animateSimulation(i));
        } else {
            modelInstance.do_data_analysis();
            console.log("simulation done in " + (performance.now() - start) / 1000 + "s")
            setUiOutput();
            setGif();
            setTimeout(() => {
                makePlots();
                makeBlockPlots();
            }, 100)
        }
    }
    let start = performance.now()
    console.log("simulation started")
    animateSimulation(0)
}

function setGif() {
    const gifData = new Uint8Array(memory.buffer, modelInstance.gif_ptr(), modelInstance.gif_len());
    let blob = new Blob([gifData], { type: "image/gif" });
    let url = URL.createObjectURL(blob);
    let img = document.getElementById("animationGif");
    img.src = url;
}

const MAIN_COLOR = "#fd7f6f";
const S_COLOR = "#7eb0d5";
const T_COLOR = "#b2e061";

function makePlots() {
    const temp = new Float32Array(memory.buffer, modelInstance.temp_ptr(), modelInstance.log_len());
    let traces = [];

    function generateAxisLayout(title) {
        return {
            color: '#FFFFFF',
            tickfont: {
                color: '#FFFFFF'
            },
            title: title,
            automargin: true,
        };
    }

    let layout = {
        autosize: true,
        responsive: true,
        plot_bgcolor: '#555',
        paper_bgcolor: '#444',
        grid: {
            rows: 3,
            columns: 2,
            pattern: 'independent'
        },
        showlegend: false,
        height: 1000,
        margin: 80
    };

    traces.push({
        x: temp,
        y: new Float32Array(memory.buffer, modelInstance.int_energy_ptr(), modelInstance.log_len()),
        line: {
            color: MAIN_COLOR
        },
        xaxis: "x",
        yaxis: "y",
    })
    layout.xaxis = generateAxisLayout("Temperature");
    layout.yaxis = generateAxisLayout("Internal Energy [pL]");

    traces.push({
        x: temp,
        y: new Float32Array(memory.buffer, modelInstance.heat_capacity_ptr(), modelInstance.log_len()),
        xaxis: "x2",
        yaxis: "y2",
        line: {
            color: MAIN_COLOR
        }
    })
    layout.xaxis2 = generateAxisLayout("Temperature");
    layout.yaxis2 = generateAxisLayout("Heat Capacity [pL]");

    traces.push({
        x: temp,
        y: new Float32Array(memory.buffer, modelInstance.acceptance_rate_ptr(), modelInstance.log_len()),
        xaxis: "x3",
        yaxis: "y3",
        line: {
            color: MAIN_COLOR
        }
    })
    layout.xaxis3 = generateAxisLayout("Temperature");
    layout.yaxis3 = generateAxisLayout("Acceptance Rate");

    traces.push({
        x: temp,
        y: new Float32Array(memory.buffer, modelInstance.entropy_ptr(), modelInstance.log_len()),
        xaxis: "x4",
        yaxis: "y4",
        line: {
            color: MAIN_COLOR
        }
    })
    layout.xaxis4 = generateAxisLayout("Temperature");
    layout.yaxis4 = generateAxisLayout("Entropy [pL]");

    traces.push({
        x: temp,
        y: new Float32Array(memory.buffer, modelInstance.free_energy_ptr(), modelInstance.log_len()),
        xaxis: "x5",
        yaxis: "y5",
        line: {
            color: MAIN_COLOR
        }
    })
    layout.xaxis5 = generateAxisLayout("Temperature");
    layout.yaxis5 = generateAxisLayout("Free Energy [pL]");

    var config = { responsive: true }
    Plotly.newPlot("plots", traces, layout, config);
}

function makeBlockPlots() {
    const temp = new Float32Array(memory.buffer, modelInstance.temp_ptr(), modelInstance.log_len());

    var config = { responsive: true }
    let layout = {
        xaxis: {
            domain: [0, 0.45],
            color: '#FFFFFF',
            tickfont: {
                color: '#FFFFFF'
            },
            title: "Temperature",
            automargin: true,
        },
        yaxis: {
            domain: [0, .9],
            color: '#FFFFFF',
            tickfont: {
                color: '#FFFFFF'
            },
            title: "Blocksize",
            automargin: true,
        },
        xaxis2: {
            domain: [0.55, 1],
            color: '#FFFFFF',
            tickfont: {
                color: '#FFFFFF'
            },
            title: "Temperature",
            automargin: true,
        },
        yaxis2: {
            anchor: 'x2',
            domain: [0, .9],
            color: '#FFFFFF',
            tickfont: {
                color: '#FFFFFF'
            },
            title: "Block Size",
            automargin: true,
        },
        autosize: true,
        responsive: true,
        title: {
            text: "Block Size Statistics",
            font: { color: "#fff" }
        },
        plot_bgcolor: '#555',
        paper_bgcolor: '#444',
        grid: {
            rows: 1,
            columns: 2,
            pattern: 'independent'
        },
        showlegend: false,
        height: 600,
        margin: 80,
        annotations: [{
            text: "Atom A",
            font: {
                size: 16,
                color: '#fff',
            },
            showarrow: false,
            align: 'center',
            x: 0.2,
            y: 1,
            xref: 'paper',
            yref: 'paper',
        },
        {
            text: "Atom B",
            font: {
                size: 16,
                color: '#fff',
            },
            showarrow: false,
            align: 'center',
            x: 0.8,
            y: 1,
            xref: 'paper',
            yref: 'paper',
        }
        ]
    };
    let traces = [];

    traces.push({
        x: temp,
        y: new Uint32Array(memory.buffer, modelInstance.cs_0_min_ptr(), modelInstance.log_len()),
        xaxis: "x1",
        yaxis: "y1",
        name: "minimum",
        line: {
            color: T_COLOR
        }
    })
    traces.push({
        x: temp,
        y: new Uint32Array(memory.buffer, modelInstance.cs_0_q1_ptr(), modelInstance.log_len()),
        xaxis: "x1",
        yaxis: "y1",
        name: "first quartile",
        line: {
            color: S_COLOR
        }
    })
    traces.push({
        x: temp,
        y: new Uint32Array(memory.buffer, modelInstance.cs_0_mean_ptr(), modelInstance.log_len()),
        xaxis: "x1",
        yaxis: "y1",
        name: "median",
        line: {
            color: MAIN_COLOR
        }
    })
    traces.push({
        x: temp,
        y: new Uint32Array(memory.buffer, modelInstance.cs_0_q3_ptr(), modelInstance.log_len()),
        xaxis: "x1",
        yaxis: "y1",
        name: "third quartile",
        line: {
            color: S_COLOR
        }
    })
    traces.push({
        x: temp,
        y: new Uint32Array(memory.buffer, modelInstance.cs_0_max_ptr(), modelInstance.log_len()),
        xaxis: "x1",
        yaxis: "y1",
        name: "maximum",
        line: {
            color: T_COLOR
        }
    })

    traces.push({
        x: temp,
        y: new Uint32Array(memory.buffer, modelInstance.cs_1_min_ptr(), modelInstance.log_len()),
        xaxis: "x2",
        yaxis: "y2",
        name: "minimum",
        line: {
            color: T_COLOR
        }
    })
    traces.push({
        x: temp,
        y: new Uint32Array(memory.buffer, modelInstance.cs_1_q1_ptr(), modelInstance.log_len()),
        xaxis: "x2",
        yaxis: "y2",
        name: "first quartile",
        line: {
            color: S_COLOR
        }
    })
    traces.push({
        x: temp,
        y: new Uint32Array(memory.buffer, modelInstance.cs_1_mean_ptr(), modelInstance.log_len()),
        xaxis: "x2",
        yaxis: "y2",
        name: "median",
        line: {
            color: MAIN_COLOR
        }
    })
    traces.push({
        x: temp,
        y: new Uint32Array(memory.buffer, modelInstance.cs_1_q3_ptr(), modelInstance.log_len()),
        xaxis: "x2",
        yaxis: "y2",
        name: "third quartile",
        line: {
            color: S_COLOR
        }
    })
    traces.push({
        x: temp,
        y: new Uint32Array(memory.buffer, modelInstance.cs_1_max_ptr(), modelInstance.log_len()),
        xaxis: "x2",
        yaxis: "y2",
        name: "maximum",
        line: {
            color: T_COLOR
        }
    })

    Plotly.newPlot("blockstats", traces, layout, config);
}

function getZip() {
    modelInstance.make_zip(method, tempSteps, startTemp, endTemp, eSteps, mSteps);
    const zipData = new Uint8Array(memory.buffer, modelInstance.get_zip_ptr(), modelInstance.get_zip_len());
    const blob = new Blob([zipData], { type: 'applications/zip' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = "thermodynamic_model.zip";
    link.click();

    URL.revokeObjectURL(url)
    modelInstance.destory_zip_data()
}


window.runSimulation = runSimulation;
window.getZip = getZip