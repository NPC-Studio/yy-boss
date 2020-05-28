/// @description Insert description here
// You can write your code in this editor

down = 270;
current_angle = 270;

gravity_vector = {
	x:0,
	y:1
};
move_vector = {
	x:1,
	y:0
}
gravity_strength = 0.5;
var _layer = layer_get_id("Collisions");
tilemap = -1;
if (layer_exists(_layer)) {
	tilemap = layer_tilemap_get_id(_layer);
}


image_speed = 0.25;
xscale = 1;
xspeed = 0;
yspeed = 4;

state = PLAYER_STATES.IDLE;

enum PLAYER_STATES {
	IDLE,
	WALK,
	RUN,
	JUMP,
	FALL
}

col_list =  ds_list_create();

finished = false;

switch_id = noone;

global.toggle = false;

ducky = false;
ducky_timer	= 10;
ducky_system = new ParticleSystem(layer,false);
ducky_particle = new ParticleType();
ducky_particle.set_sprite(spr_ducky_particle,false,false,true);
ducky_particle.set_life(room_speed,room_speed*2);
ducky_particle.set_speed(0.5,1,0.05,0.05);
ducky_particle.set_direction(80,100,0,1);
