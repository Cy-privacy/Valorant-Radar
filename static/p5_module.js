var FRAME_RATE = 45;
var TEXT_SIZE = 12;
var AGENT_ICON_SIZE = 20;

var WEAPON_ICON_WIDTH = 30;
var WEAPON_ICON_HEIGHT = 15;


var ENEMY_COLOR = 'rgb(255,0,0)';
var TEAM_COLOR = 'rgb(0,255,0)';
var CIRCLE_STROKE_COLOR = 'rgb(0,0,0)';
// unused
var flScaleX = 1.0;
var flScaleY = 1.0;

let socket;


async function get_local_json(x) {
    var response = await fetch(x)
    var data = await response.json()
    console.log(data);
    return data;
}

function DrawCircle(x, y, d, c, f) {
    if (f) { fill(c); } else { noFill(); }
    stroke(c);
    circle(x, y, d);
    noStroke();
}

function drawCustomCircle(xCenter, yCenter, diameter, fillColor, strokeColor, strokeWidth) {
    fill(fillColor);
    stroke(strokeColor);
    strokeWeight(strokeWidth);
    circle(xCenter, yCenter, diameter);
}


function DrawText(x, y, txt, txtSize, c, f) {
    textSize(txtSize);
    textFont(f);
    fill(c);
    text(txt, x, y);
}

function DrawBox(x, y, w, h, c, t, f) {
    if (f) { fill(c); } else { noFill(); }
    stroke(c);
    strokeWeight(t);
    rect(x, y, w, h);
    noStroke();
}

function DrawLine(x1, y1, x2, y2, t, c) {
    stroke(c);
    strokeWeight(t);
    line(x1, y1, x2, y2);
    noStroke();
}

function drawTriangle(x1, y1, x2, y2, x3, y3, fillColor, strokeColor, strokeWidth) {
    fill(fillColor);
    stroke(strokeColor);
    strokeWeight(strokeWidth);
    triangle(x1, y1, x2, y2, x3, y3);
    noStroke(); // Reset stroke settings
}

function rotate_point(cx, cy, x, y, angle) {
    var radians = (Math.PI / 180) * angle,
        cos = Math.cos(radians),
        sin = Math.sin(radians),
        nx = (cos * (x - cx)) + (sin * (y - cy)) + cx,
        ny = (cos * (y - cy)) - (sin * (x - cx)) + cy;
    return [nx, ny];
}

let centuryGothicFont;

var weapons_arr = [];
var agents_arr = [];

var weapon_count;
var agent_count;
function preload() {
    centuryGothicFont = loadFont('assets/fonts/Century_Gothic.ttf');
    load_agents();
    load_weapons();
}

async function load_agents() {
    var agent_data = await get_local_json('actors.json');
    agent_count = agent_data["total_agents"];
    for (var i = 0; i < agent_count; i++) {
        //console.log("assets/valorant_agents/" + agent_data["agents"][i] + ".png");
        await agents_arr.push(loadImage("assets/valorant_agents/" + agent_data["agents"][i] + ".png"));
    }
    agents_arr[0].mask(circleMask);

    console.log("Agents Loaded: " + agent_count);
}

async function load_weapons() {
    var weapon_data = await get_local_json('weapons.json');
    weapon_count = weapon_data["total_weapons"];
    for (var i = 0; i < weapon_count; i++) {
        //console.log("assets/valorant_weapons/" + weapon_data["weapons"][i] + ".png");
        await weapons_arr.push(loadImage("assets/valorant_weapons/" + weapon_data["weapons"][i] + ".png"));
    }
    console.log("Weapons Loaded: " + weapon_count);
}
var CentreX;
var CentreY;
var flMultiply = 1;
var circleMask;
var icon_size;
var text_size;
function setup() {
    //socket = new WebSocket('ws://localhost:80/ws');
    socket = new WebSocket('ws://' + window.location.host + '/ws');
    socket.onmessage = receiveData;

    var smaller_axis = Math.min(windowWidth, windowHeight);
    createCanvas(windowWidth, windowHeight);

    flScaleY = windowHeight / 1080;
    flScaleX = windowWidth / 1920;
    CentreY = windowHeight / 2;
    CentreX = windowWidth / 2;

    circleMask = createGraphics(128, 128);
    circleMask.fill('rgba(0, 0, 0, 1)');
    circleMask.circle(64, 64, 120);


    flMultiply = smaller_axis / 1024;
    if (flMultiply < 0.8) flMultiply = 0.8;
    icon_size = AGENT_ICON_SIZE * flMultiply;
    text_size = TEXT_SIZE * flMultiply;

    frameRate(FRAME_RATE);
    noSmooth();
    //console.log("Window resolution : (" + windowWidth + "x" + windowHeight + ")");
    //background('#222222');
    update_data();

}


function windowResized() {
    var smaller_axis = Math.min(windowHeight, windowWidth);
    resizeCanvas(windowWidth, windowHeight);

    flScaleY = windowHeight / 1080;
    flScaleX = windowWidth / 1920;
    CentreY = windowHeight / 2;
    CentreX = windowWidth / 2;

    flMultiply = smaller_axis / 1024;
    if (flMultiply < 0.8) flMultiply = 0.8;
    icon_size = AGENT_ICON_SIZE * flMultiply;
    text_size = TEXT_SIZE * flMultiply;

    //console.log("Window resolution : (" + windowWidth + "x" + windowHeight + ")");
    //background('#222222');
}

var data;
var last_map_name;
var map_image;
async function update_data() {

    if (socket.readyState === WebSocket.OPEN) {
        //console.log("Sending update request...");
        // socket.send('update');
    }

}
function receiveData(event) {
    //sconsole.log(event.data);
    data = JSON.parse(event.data);
}

function draw() {
    update_data();
    background('#222222');
    let fps = frameRate();
    DrawText(20, 50, "FPS: " + fps.toFixed(2), TEXT_SIZE, 'rgb(0, 255, 0)', centuryGothicFont);

    if (data == null || data == undefined || data["command"] != "render") {
        DrawText(20, 80, "Waiting for data...", TEXT_SIZE, 'rgb(0, 255, 0)', centuryGothicFont);
        return;
    }
    var entity_count = data["entity_count"];

    DrawText(20, 60, "Entities: " + entity_count, TEXT_SIZE, 'rgb(0, 255, 0)', centuryGothicFont);

    var current_map = data["map_name"];
    if (current_map != last_map_name) {
        last_map_name = current_map;
        console.log("Map changed to: " + current_map);
        map_image = loadImage("assets/valorant_maps/" + current_map + ".png");
    }


    var local_map_position_x = data["local_map_coordinate"][0];
    var local_map_position_y = data["local_map_coordinate"][1];
    var flLocalViewYaw = data["local_view_angle_y"];
    var local_agent_index = data["local_agent_index"];

    translate(CentreX, CentreY);
    rotate(PI / 180 * -flLocalViewYaw);
    scale(1.5);
    image(map_image, 512 - local_map_position_x, 512 - local_map_position_y);
    imageMode(CENTER);
    rotate(PI / 180 * flLocalViewYaw);
    drawTriangle(0, -AGENT_ICON_SIZE * 0.89 * flMultiply,
        AGENT_ICON_SIZE * flMultiply / 2.3, -AGENT_ICON_SIZE * flMultiply / 4,
        -AGENT_ICON_SIZE * flMultiply / 2.3, -AGENT_ICON_SIZE * flMultiply / 4,
        "rgb(0,0,0)", "rgb(0,0,0)",
        1
    );
    DrawCircle(0, 0, AGENT_ICON_SIZE * flMultiply, 'rgb(0,0,0)', true);
    agents_arr[local_agent_index].mask(circleMask);
    image(agents_arr[local_agent_index], 0, 0, AGENT_ICON_SIZE * flMultiply, AGENT_ICON_SIZE * flMultiply);


    for (var i = 0; i < entity_count; i++) {
        var map_position_x = data["players"][i]["map_position_x"];
        var map_position_y = data["players"][i]["map_position_y"];
        var rotated_position = rotate_point(
            0,
            0,
            map_position_x - local_map_position_x,
            map_position_y - local_map_position_y,
            flLocalViewYaw
        );
        translate(rotated_position[0], rotated_position[1]);


        var networkable = data["players"][i]["networkable"];
        var health = data["players"][i]["health"];

        //var nickname = data["players"][i]["nickname"];
        var weapon = data["players"][i]["weapon_name"];
        var is_ability = data["players"][i]["is_ability"];
        var weapon_index = data["players"][i]["weapon_index"];


        if (!networkable) {
            DrawText(
                (icon_size / 2),
                text_size / 4,
                "?",
                text_size,
                `rgb(255,255,0)`,
                true
            );
        }

        DrawText(-(icon_size / 2), (icon_size / 4) + (text_size / 2) + 5, health, text_size * 0.9, `rgb(0,255,0)`, true);

        if (is_ability == false) {
            image(weapons_arr[weapon_index], 0, (WEAPON_ICON_HEIGHT * flMultiply) + text_size, WEAPON_ICON_WIDTH * flMultiply, WEAPON_ICON_HEIGHT * flMultiply);
            //DrawText(-60, -150, weapon, 15, `rgb(0,200,255)`, true);
        }
        else {
            if (weapon == "Unknown") {
                DrawText(-2, -(icon_size / 2) - (text_size / 3), "?", text_size * 0.5, `rgb(0,180,255)`, true);

            }
            else {
                DrawText(-icon_size, -(icon_size / 2) - (text_size / 3), weapon, text_size * 0.8, `rgb(0,125,255)`, true);
            }
        }
        translate(-rotated_position[0], -rotated_position[1]);
    }

    for (var i = 0; i < entity_count; i++) {
        var map_position_x = data["players"][i]["map_position_x"];
        var map_position_y = data["players"][i]["map_position_y"];
        var rotated_position = rotate_point(
            0,
            0,
            map_position_x - local_map_position_x,
            map_position_y - local_map_position_y,
            flLocalViewYaw
        );
        var agent_index = data["players"][i]["agent_index"];
        var team = data["players"][i]["team"];

        translate(rotated_position[0], rotated_position[1]);
        rotate(PI / 180 * -flLocalViewYaw);
        rotate(PI / 180 * data["players"][i]["rotation"]["y"]);
        var triangleColor = TEAM_COLOR;
        if (team == "enemy") {
            triangleColor = ENEMY_COLOR;
        }

        drawTriangle(0, -AGENT_ICON_SIZE * 0.89 * flMultiply,
            AGENT_ICON_SIZE * flMultiply / 2.3, -AGENT_ICON_SIZE * flMultiply / 4,
            -AGENT_ICON_SIZE * flMultiply / 2.3, -AGENT_ICON_SIZE * flMultiply / 4,
            triangleColor, "rgb(0,0,0)",
            0
        );
        rotate(PI / 180 * -data["players"][i]["rotation"]["y"]);
        rotate(PI / 180 * flLocalViewYaw);

        if (health > 0) {
            if (team == "enemy") {
                DrawCircle(0, 0, AGENT_ICON_SIZE * flMultiply, ENEMY_COLOR, true);
            }
            else {
                DrawCircle(0, 0, AGENT_ICON_SIZE * flMultiply, 'rgb(0,255,0)', true);
            }
        }
        else {
            DrawCircle(0, 0, AGENT_ICON_SIZE * flMultiply, 'rgb(0,0,0)', true);
        }
        agents_arr[agent_index].mask(circleMask);
        image(agents_arr[agent_index], 0, 0, icon_size, icon_size);
        translate(-rotated_position[0], -rotated_position[1]);
    }
}


function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}
