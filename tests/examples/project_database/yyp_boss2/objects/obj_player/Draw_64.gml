/// @description Insert description here
// You can write your code in this editor
draw_set_halign(fa_left);
draw_set_valign(fa_top);
var _bbl = bbox_left+gravity_vector.x;
var _bbt = bbox_top+gravity_vector.y;
var _bbr = bbox_right+gravity_vector.x;
var _bbb = bbox_bottom+gravity_vector.y;
var _col = scr_collide_with_world(tilemap,_bbl,_bbt,_bbr,_bbb,par_block)
var _cam = obj_camera.cam;
draw_text(4,4,"x: " + string(camera_x(_cam))
			+ "\ny: " + string(camera_y(_cam))
			+ "\na: " + string(current_angle)
			+ "\nxspeed: " + string(xspeed)
			+ "\nyspeed: " + string(yspeed)
			+ "\ncol: " + string(_col)
			+ "\nfinished: " + (finished == false ? "false" : "true"));