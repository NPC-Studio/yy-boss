function __Node(_parent, _x, _y) constructor {
	
	// Register
	if (instanceof(self) == "node") throw "You cannot instantiate the node base class, please use a subclass!";
	if (_parent != undefined && instanceof(self) == "CanvasNode") throw "You must set a parent for a node!";
	nodes = [];
	if (_parent != undefined) _parent.nodes[array_length(_parent.nodes)] = self;
	
	// Values
	position = { x: _x, y: _y };
	width = 0;
	height = 0;
	scale = { x: 1, y: 1 };
	parent = _parent;
	align = { x: fa_left, y: fa_top } ;
	alpha = 1;
	color = c_white;
	angle = 0;
	truth = { position: position, width: width, height: height, scale: scale, angle: angle, color: color, alpha: alpha };
	initial = { position: position, width: width, height: height, scale: scale, angle: angle, color: color, alpha: alpha };
	bbox = { left: 0, top: 0, right: 0, bottom: 0 };
	tweens = [];
	enabled = true;
	interactable = true;
	
	#region Virtual Overrides
	
	onInputCheck = undefined;
	onInput = undefined;
	onEnabledSet = undefined;
	onInteractableSet = undefined;
	onClick = undefined;
	onHover = undefined;
	
	#endregion
	
	#region Public
	
	static registerTween = function(_tween) {
		tweens[array_length(tweens)] = _tween;
		_tween.node = self;
	}
	
	static isClicked = function() {
		if (!interactable) return false;
		if (point_in_rectangle(device_mouse_x_to_gui(0), device_mouse_y_to_gui(0), bbox.left, bbox.top, bbox.right, bbox.bottom)) { 
			if (mouse_check_button_pressed(mb_left)) return true;
		};
		return false;
	}
	
	static isHovered = function() {
		if (!interactable) return false;
		if (point_in_rectangle(device_mouse_x_to_gui(0), device_mouse_y_to_gui(0), bbox.left, bbox.top, bbox.right, bbox.bottom)) { 
			if (mouse_check_button_pressed(mb_left)) return true;
		};
		return false;
	}
	
	#endregion
	#region Private
	
	static __render = function() {};
	
	static __renderChildren = function() {
		for (var i = 0; i < array_length(nodes); i++) {
			var _node = nodes[i];
			_node.__render();
			_node.__renderChildren();
		}
	}
	
	static __updateChildren = function() {
		for (var i = 0; i < array_length(nodes); i++) {
			var _node = nodes[i];
			_node.__update();
		}
	}
	
	static __update = function() {
		if (onEnabledSet != undefined) enabled = onEnabledSet();
		if (enabled) {
			__updateAll();
			__updateTweens();
			__updateChildren();
			if (onInteractableSet != undefined) interactable = onInteractableSet();
			if (interactable) {
				if (onInputCheck != undefined && onInputCheck()) {
					if (onInput == undefined) throw "Node has an onInputCheck function but no onInput function!";
					onInput();
				}
			}
		}
	}
	
	static __updateTweens = function() {
		for (var i = 0; i < array_length(tweens); i++) {
			var _tween = tweens[i];
			_tween.__update();
		}
	}
		
	static __updateCustom = function () {};

	static __updateScale = function() {
		truth.scale = {
			x: scale.x * parent.truth.scale.x / parent.initial.scale.x,
			y: scale.y * parent.truth.scale.y / parent.initial.scale.y
		};
	}
	
	static __updateWidthHeight = function() {
		truth.width = width * truth.scale.x;
		truth.height = height * truth.scale.y;
	}
	
	static __updateAlpha = function() {
		if (parent.initial.alpha != 0) { // god forbid we divide by zero
			truth.alpha = alpha * parent.truth.alpha / parent.initial.alpha;	
		} else {
			truth.alpha = alpha * parent.truth.alpha;
		}
	}
	
	static __updateColor = function() {
		truth.color = color != c_white ? merge_color(color, parent.truth.color, 0.5) : parent.truth.color;
	}
	
	static __updateAngle = function() {
		truth.angle = parent.truth.angle + angle;	
	}
	
	static __updatePosition = function() {
		// Calculate relative values (based on scale)
		truth.position = { x: position.x * abs(parent.truth.scale.x) / abs(parent.initial.scale.x), y: position.y * abs(parent.truth.scale.y) / abs(parent.initial.scale.y) };
		switch (align.x) {
			case fa_left: truth.position.x += parent.truth.position.x; break;
			case fa_center: truth.position.x += (parent.truth.position.x + (parent.truth.width / 2)) - (truth.width / 2); break;
			case fa_right: truth.position.x += (parent.truth.position.x + parent.truth.width) - truth.width; break;
		}
		switch (align.y) {
			case fa_top: truth.position.y += parent.truth.position.y; break;
			case fa_middle: truth.position.y += (parent.truth.position.y + (parent.truth.height / 2)) - (truth.height / 2); break;
			case fa_bottom: truth.position.y += (parent.truth.position.y + parent.truth.height) - truth.height; break;
		}
	}
	
	static __updateBBox = function() {
		bbox.left = truth.position.x;
		bbox.top = truth.position.y;
		bbox.right = truth.position.x + truth.width;
		bbox.bottom = truth.position.y + truth.height;
	}
	
	static __updateAll = function() {
		__updateScale();
		__updateWidthHeight();
		__updateAlpha();
		__updateAngle();
		__updateColor();
		__updatePosition();
		__updateBBox();
		__updateCustom();
	}

	#endregion
}

function CanvasNode(_parent, _x, _y) : __Node(_parent, _x, _y) constructor {
	width = display_get_gui_width();
	height = display_get_gui_height();
	truth.width = width;
	truth.height = height;
	
	// Overwrite calculation (this thing don't move!)
	/*override*/ static __updateAll = function() {};
}

function TextNode(_parent, _x, _y, _text) : __Node(_parent, _x, _y) constructor {
	text = _text;
	font = -1; // GM's default Arial 12 point
	lineHeight = 21; // This is rather arbitrarily what GM seems to like for the default
	maxWidth = undefined;
	canLineBreak = true;
	truth.maxWidth = undefined;
	__textCache = undefined;
	__surface = noone;
	
	onTextProcess = undefined;
	
	/*override*/ static __render = function() {
		if (text != __textCache) {
			if (onTextProcess != undefined) text = onTextProcess(text);
			__calculateSize();
			if (surface_exists(__surface)) surface_free(__surface);
			__textCache = text;
		}
		if (global.anchorConfig.surfaceBaking) {
			if (!surface_exists(__surface)) {
				__surfaceBake();
				__updatePosition();
			}
			draw_surface_ext(__surface, truth.position.x, truth.position.y, truth.scale.x, truth.scale.y, truth.angle, truth.color, truth.alpha);
		} else {
			__renderRaw(truth.position.x, truth.position.y);
		}
	}
	
	/*override*/ static __updateWidthHeight = function() {
		// does nothing, as width is set in __calculateSize
	}
	
	static __renderRaw = function(_x, _y) {
		var _cache = {
			font: draw_get_font(),
			halign: draw_get_halign(),
			valign: draw_get_valign()
		};
		draw_set_font(font);
		draw_set_halign(align.x);
		draw_set_valign(align.y);
		draw_set_color(truth.color);
		gpu_set_blendenable(false);
		draw_text_ext_transformed(_x, _y, text, lineHeight, truth.maxWidth, 1, 1, 0);
		gpu_set_blendenable(true);
		draw_set_color(c_white);
		draw_set_font(_cache.font);
		draw_set_halign(_cache.halign);
		draw_set_valign(_cache.valign);
	}
	
	static __calculateSize = function() {
		// We use 0.9 to ensure that text never touches the edges of a textbox.
		// This can be changed to fit your preferences!
		truth.maxWidth = maxWidth != undefined ? maxWidth : ((parent.truth.width * parent.initial.scale.x) * 0.9) / scale.x;
		if !(canLineBreak) truth.maxWidth = 10000000; // I just don't trust GM's infinity, man
		var _cache = {
			font: draw_get_font(),
			halign: draw_get_halign(),
			valign: draw_get_valign()
		};
		draw_set_font(font);
		draw_set_halign(align.x);
		draw_set_valign(align.y);
		truth.width = string_width_ext(text, lineHeight, truth.maxWidth);
		truth.height = string_height_ext(text, lineHeight, truth.maxWidth);
		draw_set_font(_cache.font);
		draw_set_halign(_cache.halign);
		draw_set_valign(_cache.valign);
	}

	static __surfaceBake = function() {
		if (surface_exists(__surface)) surface_free(__surface);
		if (text == "") {
			show_debug_message("ANCHOR WARNING: Anchor does not support empty strings on text nodes when using baking. Setting string to ' '.");
			text = " ";
		}
		__surface = surface_create(truth.width, truth.height);
		surface_set_target(__surface);
		var _textPosition = { x: 0, y: 0 };
		switch (align.x) {
			case fa_left: _textPosition.x = 0; break;
			case fa_center: _textPosition.x = truth.width / 2; break;
			case fa_right: _textPosition.x = truth.width; break;
		}
		switch (align.y) {
			case fa_top: _textPosition.y = 0; break;
			case fa_middle: _textPosition.y = truth.height / 2; break;
			case fa_bottom: _textPosition.y = truth.height; break;
		}
		__renderRaw(_textPosition.x, _textPosition.y);
		surface_reset_target();
	}
}

function BoxNode(_parent, _x, _y, _width, _height, _sprite) : __Node(_parent, _x, _y) constructor {
	width = _width;
	height = _height;
	sprite = _sprite;
	__surface = noone;
	
	/*override*/ static __render = function() {
		if !(surface_exists(__surface)) {
			__surfaceBake();
		}
		draw_surface_ext(__surface, truth.position.x, truth.position.y, truth.scale.x, truth.scale.y, truth.angle, truth.color, truth.alpha);
	}
	
	static __surfaceBake = function() {
		if (surface_exists(__surface)) surface_free(__surface);
		__surface = surface_create(truth.width, truth.height);
		surface_set_target(__surface);
		var _lil = sprite_get_width(sprite) / 3;
		var _bigl = _lil * 2;
		draw_set_alpha(truth.alpha);
		draw_set_color(truth.color);
		draw_sprite_part_ext(sprite, 0, _lil, _lil, 1, 1, 0 + _lil, 0 + _lil, width - _bigl, height - _bigl, truth.color, truth.alpha);
		draw_sprite_part(sprite, 0, 0, 0, _lil, _lil, 0, 0);
		draw_sprite_part(sprite, 0, _bigl, 0, _lil, _lil, width - _lil, 0);
		draw_sprite_part(sprite, 0, 0, _bigl, _lil, _lil, 0, height - _lil);
		draw_sprite_part(sprite, 0, _bigl, _bigl, _lil, _lil, width - _lil, height - _lil);
		draw_sprite_part_ext(sprite, 0, 0, _lil, _lil, 1, 0, 0 + _lil, 1, height - _bigl, truth.color, truth.alpha);
		draw_sprite_part_ext(sprite, 0, _bigl, _lil, _lil, 1, width - _lil, 0 + _lil, 1, height - _bigl, truth.color, truth.alpha);
		draw_sprite_part_ext(sprite, 0, _lil, 0, 1, _lil, 0 + _lil, 0, width - _bigl, 1, truth.color, truth.alpha);
		draw_sprite_part_ext(sprite, 0, _lil, _bigl, 1, _lil, 0 + _lil, height - _lil, width - _bigl, 1, truth.color, truth.alpha);
		draw_set_color(c_white);
		draw_set_alpha(1);
		surface_reset_target();
	}
}

function SpriteNode(_parent, _x, _y, _sprite) : __Node(_parent, _x, _y) constructor {
	sprite = _sprite;
	imageIndex = 0;
	imageSpeed = 1;
	truth.imageSpeed = 1;
	
	/*override*/ static __updateWidthHeight = function() {
		truth.width = sprite_get_width(sprite) * truth.scale.x;
		truth.height = sprite_get_height(sprite) * truth.scale.y;
	}
	
	/*override*/ static __updateCustom = function() {
		// Make sure the position is based on a top-left coordinate of the sprite
		truth.position.x += sprite_get_xoffset(sprite) * truth.scale.x;
		truth.position.y += sprite_get_yoffset(sprite) * truth.scale.y;
		
		// Animatiion
		var _speed = sprite_get_speed(sprite);
		truth.imageSpeed = sprite_get_speed_type(sprite) == spritespeed_framespersecond ? (_speed / 60) * imageSpeed : _speed * imageSpeed;
		imageIndex += truth.imageSpeed;
		if (imageIndex > sprite_get_number(sprite)) {
			imageIndex = 0;
		}
	}
	
	/*override*/ static __updateBBox = function() {
		bbox.left = truth.position.x + (sprite_get_bbox_left(sprite) * abs(truth.scale.x));
		bbox.top = truth.position.y + (sprite_get_bbox_top(sprite) * abs(truth.scale.y));
		bbox.right = truth.position.x + (sprite_get_bbox_right(sprite) * abs(truth.scale.x));
		bbox.bottom = truth.position.y + (sprite_get_bbox_bottom(sprite) * abs(truth.scale.y));
	}
	
	/*override*/ static __render = function() {
		draw_sprite_ext(sprite, imageIndex, truth.position.x, truth.position.y, truth.scale.x, truth.scale.y, truth.angle, truth.color, truth.alpha);	
	}
}