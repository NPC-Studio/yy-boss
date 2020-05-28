/// @description Insert description here
// You can write your code in this editor

if (state != last_state) {
	switch_state();
}

if (btn_login_option != noone && btn_login_option.pressed) {
	state = 1;
}

if (btn_register_option != noone && btn_register_option.pressed) {
	state = 2;
}

if (btn_login != noone && btn_login.pressed) {
	obj_online_controller.login(txt_email.content,txt_password.content);
}

if (btn_register != noone && btn_register.pressed) {
	obj_online_controller.register(txt_name.content,txt_email.content,txt_password.content);
}

if (btn_start.pressed) {
	room_goto(rm_test);	
}

if ((obj_online_controller.state == ONLINE_STATES.LOGGED_IN) && (state != 3)) {
	show_message("Logged in")
	state = 3;	
}