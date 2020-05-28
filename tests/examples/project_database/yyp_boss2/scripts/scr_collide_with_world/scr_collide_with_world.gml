// Script assets have changed for v2.3.0 see
// https://help.yoyogames.com/hc/en-us/articles/360005277377 for more information
function scr_collide_with_world(_tilemap,_x1,_y1,_x2,_y2,_obj){
	///@desc scr_collide_with_world
	///@arg {real} tilemap
	///@arg {real} x1
	///@arg {real} y1
	///@arg {real} x2
	///@arg {real} y2
	///@arg {object} box_parent

	var _col = false;
	if (_tilemap != -1) {		//returns -1 on failure
		_col = scr_tilemap_box_collision(_tilemap, _x1, _y1, _x2, _y2);
	}
	if (_col) {
		return (_col);
	}

	var _w = _x2-_x1;

	var _inst = collision_rectangle(_x1,_y1,_x2,_y2,_obj,false,true);
	if (instance_exists(_inst)) {		//early exit
		return _inst;
	}
	ds_list_clear(col_list);

	return _inst;

}