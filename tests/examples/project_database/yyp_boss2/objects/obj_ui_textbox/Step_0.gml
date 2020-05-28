/// @description Insert description here

event_inherited();
if (focused==true) {
	if (verify_function == undefined) {
		content += keyboard_string;
	} else {
		if (verify_function(content + keyboard_string)) {
			content += keyboard_string;	
		}
	}
	keyboard_string = "";
	
	if (keyboard_check(vk_backspace)) {
		io_clear();
		content = string_delete(content,string_length(content),1);
	}
	w = lerp(w,1,lerp_speed);
} else {
	w = lerp(w,0,lerp_speed);
}
if (mouse_check_button_pressed(mb_left)) {
	if (point_in_rectangle(mouse_x,mouse_y,x,y,x+width,y+height)) {
		with (par_control) { 
			focused = false;
		}
		if (focused = false) {
			w = 0;
			focused = true;
		}
		
		keyboard_string = "";
	}
}