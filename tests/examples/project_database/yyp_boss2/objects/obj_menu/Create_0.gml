/// @description 

txt_name = noone; //textbox_add(-100,-100,64,20);
txt_email = noone; //textbox_add(-100,-100,64,20);
txt_password = noone; //textbox_add(-100,-100,64,20);
chk_remember_me = noone;
btn_login = noone;
btn_register = noone;
btn_login_option = noone;
btn_register_option = noone;

state = 0;
last_state = 0;

//chk_remember_me = checkbox_add(32,32,16,16,{checked:true,text:"Remember login?",align:UI_ALIGN.RIGHT})
//btn_login_option = button_add(32,32,96,20,{text:"Log In",callback: function() { 
//    obj_menu.state = 1;
//    show_message("._.");
//}});

/**/

btn_start = button_add(4,room_height-128,192,20,{text:"Start Game",callback: undefined});
btn_browse = button_add(4,room_height-104,192,20,{text:"Browse Levels",callback: undefined});
btn_editor = button_add(4,room_height-80,192,20,{text:"Level Editor",callback: undefined});
btn_options = button_add(4,room_height-56,192,20,{text:"Options",callback: undefined});
btn_exit = button_add(4,room_height-32,192,20,{text:"Exit",callback: undefined});

function switch_state() {
	instance_destroy(txt_name);
	instance_destroy(txt_email);
	instance_destroy(txt_password);
	instance_destroy(btn_login);
	instance_destroy(btn_register);
	instance_destroy(btn_login_option);
	instance_destroy(btn_register_option);
	instance_destroy(chk_remember_me);
	
	txt_name = noone;
	txt_email = noone;
	txt_password = noone;
	btn_login = noone;
	btn_register = noone;
	btn_login_option = noone;
	btn_register_option = noone;
	chk_remember_me = noone;
	
	last_state = state;
	
	switch (state) {
		case 0:
			btn_login_option = button_add(16,32,192,20,{text:"Log into account",callback: undefined});
			btn_register_option = button_add(16,64,192,20,{text:"Register an account",callback: undefined});
		break;
		case 1:
			txt_email = textbox_add(16,16,192,20,"text",{ label: "Email" });
			txt_password = textbox_add(16,40,192,20,"password",{ label: "Password" });
			btn_login = button_add(16,112,192,20,{text:"Log In",callback: undefined});
		break;
		case 2:
			txt_name = textbox_add(16,16,192,20,"text",{ label: "Name" });
			txt_email = textbox_add(16,40,192,20,"text",{ label: "Email" });
			txt_password = textbox_add(16,64,192,20,"password",{ label: "Password" });
			btn_register = button_add(16,112,192,20,{text:"Register",callback: undefined});
		break;
	}
}

switch_state();

