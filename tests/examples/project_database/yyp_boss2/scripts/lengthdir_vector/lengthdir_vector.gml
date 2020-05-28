// Script assets have changed for v2.3.0 see
// https://help.yoyogames.com/hc/en-us/articles/360005277377 for more information
function lengthdir_vector(len,dir){
	return {
		x: lengthdir_x(len,dir),
		y: lengthdir_y(len,dir)
	}
}