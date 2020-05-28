/// @description Insert description here
// You can write your code in this editor

if (global.toggle == toggle_when) {
	with (instance_create_layer(x,y,layer,previous_object)) {
		sprite_index = other.sprite_index;
		image_blend = c_orange;
		image_xscale = other.image_xscale;
		image_yscale = other.image_yscale;
		
		toggle_when = !other.toggle_when;	
	}
	instance_destroy();
}