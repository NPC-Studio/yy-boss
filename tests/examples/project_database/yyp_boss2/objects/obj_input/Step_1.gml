/// @description Process inputs!

var _map = global.options;

input_refresh_begin();

//Note that we can stack many different inputs for the same input slot
//This system supports gamepad axis, gamepad button, keyboard button, and mouse button input

input_handle_gamepad_axis(0,E_INPUT_SLOT.UP,gamepad_device,gp_axislv,-0.3);
input_handle_gamepad_button(0,E_INPUT_SLOT.UP,gamepad_device,gp_padu);
input_handle_keyboard(0,E_INPUT_SLOT.UP,vk_up, ord("W"));

input_handle_gamepad_axis(0,E_INPUT_SLOT.DOWN,gamepad_device,gp_axislv,0.3);
input_handle_gamepad_button(0,E_INPUT_SLOT.DOWN,gamepad_device,gp_padd);
input_handle_keyboard(0,E_INPUT_SLOT.DOWN,vk_down,ord("S"));

input_handle_gamepad_axis(0,E_INPUT_SLOT.LEFT,gamepad_device,gp_axislh,-0.3);
input_handle_gamepad_button(0,E_INPUT_SLOT.LEFT,gamepad_device, _map[? "gp_left"]);
input_handle_keyboard(0,E_INPUT_SLOT.LEFT,vk_left,ord("A"));

input_handle_gamepad_axis(0, E_INPUT_SLOT.RIGHT,gamepad_device,gp_axislh,0.3);
input_handle_gamepad_button(0,E_INPUT_SLOT.RIGHT,gamepad_device,_map[? "gp_right"]);
input_handle_keyboard(0,E_INPUT_SLOT.RIGHT,_map[? "key_right"]);

input_handle_gamepad_button(0,E_INPUT_SLOT.JUMP,gamepad_device,_map[? "gp_jump"]);
input_handle_keyboard(0,E_INPUT_SLOT.JUMP,vk_space,vk_enter,_map[? "key_jump"]);

input_refresh_end(repeat_delay,longpress_delay,doubletap_delay); //Actual state processing happens in this script
