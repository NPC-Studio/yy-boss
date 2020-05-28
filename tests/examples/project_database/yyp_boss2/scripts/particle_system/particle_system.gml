// Script assets have changed for v2.3.0 see
// https://help.yoyogames.com/hc/en-us/articles/360005277377 for more information
function ParticleSystem(_layer,_persistent) constructor {
	system = part_system_create_layer(_layer,_persistent);
	__update = true;
	__draw = true;
	__old_to_new = true;
	function reset() {
		part_system_clear(system);	
	}
	function clear() {
		part_particles_clear(system);
	}
	function automatic_update(_update) {
		__update = _update;
		part_system_automatic_update(system,_update);
	}
	function get_automatic_update() {
		return __update;	
	}
	function get_automatic_draw() {
		return __draw;	
	}
	function automatic_draw(_draw) {
		__draw = _draw;
		part_system_automatic_draw(system,_draw);
	}
	function draw_order(old_to_new) {
		__old_to_new = old_to_new;
		part_system_draw_order(system,old_to_new)	
	}
	function count() {
		return part_particles_count(system);	
	}
	function create(_x,_y,particle,num) {
		if (is_struct(particle)) { // custom particle struct
			particle = particle.particle;
		}
		part_particles_create(system,_x,_y,particle,num);
	}
	function update() {
		part_system_update(system);	
	}
	function draw() {
		part_system_drawit(system);	
	}
	function destroy() {
		part_system_destroy(system);	
		system = undefined;
	}
}

function ParticleType() constructor {
	particle = part_type_create();
	__sprite = noone;
	__animated = noone;
	__stretch = noone;
	__random = noone;
	__shape = noone;
	__size_min = 1;
	__size_max = 1;
	__size_increase = 0;
	__size_wiggle = 0;
	__scale = 1;
	__color = [c_white];
	__alpha = [1];
	__xscale = 1;
	__yscale = 1;
	__speed = 0;
	__speed_min	= 0;
	__speed_max	= 0;
	__speed_increase = 0;
	__speed_wiggle = 0;
	__gravity_strength = 0;
	__gravity_direction = 0;
	__life_min = 1;
	__life_max = 1;
	__angle_min = 0;
	__angle_max = 0;
	__angle_increase = 0;
	__angle_wiggle = 0;
	__angle_relative = 0;
	__blend = false;
	function set_sprite(_sprite,_animated,_stretch,_random) {
		__sprite = _sprite;
		__animated = _animated;
		__stretch = _stretch;
		__random = _random;
		part_type_sprite(particle,_sprite,_animated,_stretch,_random);	
	}
	function set_shape(shape) {
		__shape = shape;
		part_type_shape(particle,shape);	
	}
	function set_size(_size_min,_size_max,_size_increase,_size_wiggle) {
		__size_min = _size_min;
		__size_max = _size_max;
		__size_increase = _size_increase;
		__size_wiggle = _size_wiggle;
		part_type_size(particle,_size_min,_size_max,_size_increase,_size_wiggle);
	}
	function set_scale(_xscale,_yscale) {
		__xscale = _xscale;
		__yscale = _yscale;
		part_type_scale(particle,_xscale,_yscale);
	}
	function set_speed(_speed_min,_speed_max,_speed_increase,_speed_wiggle) {
		__speed_min = _speed_min;
		__speed_max = _speed_max;
		__speed_increase = _speed_increase;
		__speed_wiggle = _speed_wiggle;
		part_type_speed(particle,_speed_min,_speed_max,_speed_increase,_speed_wiggle);
	}
	function set_direction(_direction_min,_direction_max,_direction_increase,_direction_wiggle) {
		__direction_min = _direction_min;
		__direction_max = _direction_max;
		__direction_increase = _direction_increase;
		__direction_wiggle = _direction_wiggle;
		part_type_direction(particle,_direction_min,_direction_max,_direction_increase,_direction_wiggle);
	}
	function set_gravity(_gravity_strength,_gravity_direction) {
		__gravity_strength = _gravity_strength;
		__gravity_direction = _gravity_direction;
		part_type_gravity(particle,_gravity_strength,_gravity_direction)	
	}
	function set_orientation(_angle_min,_angle_max,_angle_increase,_angle_wiggle,_angle_relative) {
		__angle_min = _angle_min;
		__angle_max = _angle_max;
		__angle_increase = _angle_increase;
		__angle_wiggle = _angle_wiggle;
		__angle_relative = _angle_relative;
		part_type_orientation(particle,_angle_min,_angle_max,_angle_increase,_angle_wiggle,_angle_relative);
	}
	function set_color() {
		__color = [];
		for (var i=0;i<arguent_count;i++) {
			__color[i] = argument[i];
		}
		switch (array_length(__color)) {
			case 1:
				part_type_color1(particle,__color[0]);
			break;
			case 2:
				part_type_color2(particle,__color[0],__color[1]);
			break;
			case 3:
				part_type_color3(particle,__color[0],__color[1],__color[2]);
			break;
		}
	}
	function set_alpha() {
		__alpha = [];
		for (var i=0;i<arguent_count;i++) {
			__alpha[i] = argument[i];
		}
		switch (array_length(__alpha)) {
			case 1:
				part_type_alpha1(particle,__alpha[0]);
			break;
			case 2:
				part_type_alpha2(particle,__alpha[0],__alpha[1]);
			break;
			case 3:
				part_type_alpha3(particle,__alpha[0],__alpha[1],__alpha[2]);
			break;
		}
	}
	function set_life(_life_min,_life_max) {
		__life_min = _life_min;
		__life_max = _life_max;
		part_type_life(particle,_life_min,_life_max);
	}
	function set_blend(_blend) {
		__blend = _blend;
		part_type_blend(particle,_blend);
	}
	function get_shape() {
		return __shape;	
	}
	function get_sprite() {
		return __sprite;
	}
	function get_sprite_animated() {
		return __animated;
	}
	function get_sprite_stretch() {
		return __stretch;
	}
	function get_sprite_random() {
		return __random;
	}
	function get_size_min() {
		return __size_min;
	}
	function get_size_max() {
		return __size_max;
	}
	function get_size_increase() {
		return __size_increase;
	}
	function get_size_wiggle() {
		return __size_wiggle;
	}
	function get_xscale() {
		return __xscale;
	}
	function get_yscale() {
		return __yscale;
	}
	function get_speed_min() {
		return __speed_min;
	}
	function get_speed_max() {
		return __speed_max;
	}
	function get_speed_increase() {
		return __speed_increase;
	}
	function get_speed_wiggle() {
		return __speed_wiggle;
	}
	function get_gravity_strength() {
		return __gravity_strength;	
	}
	function get_gravity_direction() {
		return __gravity_direction;
	}
	function get_direction_min() {
		return __direction_min;
	}
	function get_direction_max() {
		return __direction_max;
	}
	function get_direction_increase() {
		return __direction_increase;
	}
	function get_direction_wiggle() {
		return __direction_wiggle;
	}
	function get_angle_min() {
		return __angle_min;
	}
	function get_angle_max() {
		return __angle_max;
	}
	function get_angle_increase() {
		return __angle_increase;
	}
	function get_angle_wiggle() {
		return __angle_wiggle;
	}
	function get_color() {
		return __color;	
	}
	function get_alpha() {
		return __alpha;	
	}
	function get_blend() {
		return __blend;	
	}
}