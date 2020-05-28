/// @description Insert description here
// You can write your code in this editor

// Inherit the parent event
event_inherited();
if (point_in_rectangle(mouse_x,mouse_y,x,y,x+width,y+height)) {
	if (mouse_check_button_pressed(mb_left)) {
		with (par_control) { 
			focused = false;
		}
		if (focused = false) {
			focused = true;
		}
		mouse_in = true;
	}
	if (mouse_check_button_released(mb_left)) {
		if (mouse_in) {
			mouse_in = false;
			checked = !checked;
		}
	}
} else {
	if (!mouse_check_button(mb_left)) {
		mouse_in = false;	
	}
}