/// @description Insert description here
// You can write your code in this editor
instance_create_depth(0,0,0,obj_camera);
global.options = ds_map_create();
var _map = global.options;

_map[? "key_left"] = ord("A");
_map[? "key_right"] = ord("D");
_map[? "key_jump"] = vk_space;
_map[? "gp_left"] = gp_padl;
_map[? "gp_right"] = gp_padr;
_map[? "gp_jump"] = gp_face1;

if (file_exists("options.json")) {
	var _buff = buffer_load("options.json");
	var _opt = json_decode(buffer_read(_buff,buffer_text));
	buffer_delete(_buff);
	var _key = ds_map_find_first(_opt);
	while (_key != undefined) {
		global.options[? _key] = _opt[? _key];
		_key = ds_map_find_next(_opt,_key);
	}
	ds_map_destroy(_opt);
}
room_goto_next();

