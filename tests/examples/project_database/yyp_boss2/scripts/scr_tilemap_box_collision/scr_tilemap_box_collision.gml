// Script assets have changed for v2.3.0 see
// https://help.yoyogames.com/hc/en-us/articles/360005277377 for more information
function scr_tilemap_box_collision(_tilemap,_x1,_y1,_x2,_y2){
	/// @desc scr_tilemap_box_collision
	/// @arg tilemap
	/// @arg x1
	/// @arg y1
	/// @arg x2
	/// @arg y2

	var _p = [];

	var _w = _x2-_x1;
	var _h = _y2-_y1;
	var _x = 0;
	var _y = 0;
	var _i = 0;

	while (_x <= _w) {
	
		_p[_i] = tilemap_get_at_pixel(_tilemap,_x1+_x,_y1);
		_i++;
		_p[_i] = tilemap_get_at_pixel(_tilemap,_x1+_x,_y2);
		_i++;
		_x+=32;	
	}
	while (_y <= _h) {
	
		_p[_i] = tilemap_get_at_pixel(_tilemap,_x1,_y1+_y);
		_i++;
		_p[_i] = tilemap_get_at_pixel(_tilemap,_x2,_y1+_y);
		_i++;
		_y+=32;	
	}

	var _i = 0, _s = 0, _length = array_length(_p);
	while (_i < _length) {
		_s = (_p[_i++] > 0) || _s;
	}
	return (_s);

}