/// @description Insert description here
// You can write your code in this editor
var _cam = obj_camera.cam;
var _w = obj_camera.width;
var _h = obj_camera.height;

var _left = global.options[? "key_left"];
var _right = global.options[? "key_right"];
var _jump_key = global.options[? "key_jump"];

if (keyboard_check_pressed(vk_f2)) {
	ducky = !ducky;	
}

if (finished) {
	
	if (keyboard_check_pressed(ord("R"))) {
		room_restart();
	}
	return;	
}
var _lr = input_check(0,E_INPUT_SLOT.RIGHT)-input_check(0,E_INPUT_SLOT.LEFT);
var _jump = input_check(0,E_INPUT_SLOT.JUMP);

if (down < 0) {
	while (down < 0) {
		down += 360;	
	}
}
down = down mod 360;

if (current_angle < 0) {
	while (current_angle < 0) {
		current_angle += 360;	
	}
}
current_angle = current_angle mod 360;

var _dp = down;
down -= (keyboard_check_pressed(vk_left)-keyboard_check_pressed(vk_right))*90;
var _inst = instance_place(x,y,obj_arrow_parent);
if (_inst != noone) {
	down = -_inst.dir + 180;	
}

if (down != _dp) {
	//var _nx = dcos(down)*xspeed-(yspeed*dsin(down));
	//var _ny =-dsin(down)*yspeed-(xspeed*dcos(down));;
	//xspeed = _nx;
	//yspeed = _ny;
	var _nx, _ny;
	var _diff = angle_difference(down,_dp);
	switch (_diff) {
		case 0:
			_nx = xspeed;
			_ny = yspeed;
		break;
		case 90:
			_nx = yspeed;
			_ny =-xspeed;
		break;
		case -180:
		case 180:
			_nx =-xspeed;
			_ny =-yspeed;
		break;
		case -90:
			_nx =-yspeed;
			_ny = xspeed;
		break;
	}
	xspeed = _nx;
	yspeed = _ny;
}


gravity_vector = lengthdir_vector(1,-down+180);
move_vector = lengthdir_vector(1,-down+180+90);


//x += (keyboard_check(ord("D")) - keyboard_check(ord("A")))*4;
//y += (keyboard_check(ord("S")) - keyboard_check(ord("W")))*4;





if (abs(_lr)) {
	xscale = sign(_lr);
	xspeed += _lr;
	xspeed = clamp(xspeed,-8,8);
}



if (_lr) {
	xspeed += _lr
}

var _max_xspeed = 8;
if (keyboard_check(vk_shift)) {
	_max_xspeed = 4;
}

xspeed = clamp(xspeed,-_max_xspeed,_max_xspeed);
xspeed = max(0,abs(xspeed - (0.5*sign(xspeed))))*sign(xspeed);
yspeed += gravity_strength;	
if (place_meeting(x+gravity_vector.x,y+gravity_vector.y,par_block)) {	
	yspeed = 0;
}
if (_jump && place_meeting(x+gravity_vector.x,y+gravity_vector.y,par_block)) {//scr_collide_with_world(tilemap,_bbl,_bbt,_bbr,_bbb,par_block)) {
	yspeed = -8;
}

if (_jump && yspeed < -4) {
	yspeed -= 0.25;	
}

var _xs = sign(xspeed);
var _xsp = xspeed div 1;
repeat (abs(_xsp)) {
	if (place_meeting(x+move_vector.x*_xs,y+move_vector.y*_xs,par_block)) {
		break;
	} else {
		x += move_vector.x * _xs;
		y += move_vector.y * _xs;
	}
}

var _ys = sign(yspeed);
var _ysp = yspeed div 1;
repeat (abs(_ysp)) {
	if (place_meeting(x+gravity_vector.x*_ys,y+gravity_vector.y*_ys,par_block)) {
		yspeed = 0;
		break;
	} else {
		x += gravity_vector.x * _ys;
		y += gravity_vector.y * _ys;
	}
}

// mechanics

if (place_meeting(x,y,obj_bound)) {
	// out of bounds - stub for later
	room_restart();
}

if (place_meeting(x,y,obj_goal)) {
	if (finished == false) {
		// stubbiest of all stubs
	}
	finished = true;	
}

var _inst = instance_place(x,y,obj_switch);
if (_inst != switch_id) {	
	switch_id = _inst;
	if (_inst != noone) {
		global.toggle = !global.toggle;
	}
}

#region animation

if (place_meeting(x+gravity_vector.x,y+gravity_vector.y,par_block)) {
	// on ground	
	if (xspeed == 0) {
		state = PLAYER_STATES.IDLE;
	} else if (abs(xspeed) < 5) {
		state = PLAYER_STATES.WALK;
	} else {
		state = PLAYER_STATES.RUN;
	}
} else {
	 if (yspeed < 0) {
		state = PLAYER_STATES.JUMP;	 
	 } else {
		state = PLAYER_STATES.FALL;	 
	 }
}

switch (state) {
	case PLAYER_STATES.IDLE:
		sprite_index = spr_rsg_stand;
	break;	
	case PLAYER_STATES.WALK:
		sprite_index = spr_rsg_walk;
	break;
	case PLAYER_STATES.RUN:
		sprite_index = spr_rsg_run;
	break;
	case PLAYER_STATES.JUMP:
	case PLAYER_STATES.FALL:
		sprite_index = spr_rsg_jump;
		
		var _frame = (yspeed div 1)/2;
		image_index = clamp((image_number/2) + _frame,0,image_number-1);
	break;
}
if (ducky) {
	sprite_index = spr_ducky;
	ducky_particle.set_direction(-down-10,-down+10,0,1);
	ducky_particle.set_orientation(270-down-10,270-down+10,0,1,false);
	if (ducky_timer-- <= 0) {
		ducky_system.create(x + random_range(-32,32),y+random_range(-32,32),ducky_particle,1);
		ducky_timer = 10;
	}
}
#endregion



#region camera

var _c = dcos(down);
var _s = dsin(down);

var _xoff = 0;
var _yoff = 0;
var _cx = x + _c * _xoff * 1 + _s * _yoff * 1 - _w / 2;
var _cy = y + _c * _yoff * 1 - _s * _xoff * 1 - _h / 2;

current_angle = lerp(current_angle,current_angle+angle_difference(down,current_angle),0.125);

camera_x(_cam,_cx);
camera_y(_cam,_cy);
camera_set_view_angle(obj_camera.cam,(current_angle+90));

#endregion