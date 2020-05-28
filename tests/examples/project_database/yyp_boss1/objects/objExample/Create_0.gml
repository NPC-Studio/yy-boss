/// @desc 

// Inherit the parent event
event_inherited();

box = new BoxNode(canvas, 0, 0, 80, 100, sprBox);
box.align = { x: fa_center, y: fa_middle };
box.scale = { x: 3, y: 3 };

sprite = new SpriteNode(box, 0, 0, sprSprite);
sprite.align = { x: fa_center, y: fa_middle };
sprite.onInputCheck = function() {
	return sprite.isClicked();	
}
sprite.onInput = function() {
	show_debug_message("I've been clicked!");
	sprite.color = irandom(c_white);
}