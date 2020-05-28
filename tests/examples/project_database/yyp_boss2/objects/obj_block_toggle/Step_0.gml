/// @description Insert description here
// You can write your code in this editor
if (global.toggle == toggle_when) {
	with (instance_create_layer(x,y,layer,obj_toggle_off)) {
		sprite_index = other.sprite_index;
		image_alpha = 0.5;
		image_blend = c_orange;
		image_xscale = other.image_xscale;
		image_yscale = other.image_yscale;
		
		previous_object = other.object_index;
		toggle_when = !other.toggle_when;
	}
	instance_destroy();
}