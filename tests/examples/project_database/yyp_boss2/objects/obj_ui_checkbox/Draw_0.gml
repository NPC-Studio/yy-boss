/// @description Insert description here

draw_set_color(c_accent);
draw_rectangle(x,y,x+width,y+height,true);

var _cx = x + width/2;
var _cy = y + height/2;
var _ls = 0.25;


if (checked) {
	lcw = lerp(lcw,(width/2 - 2),_ls);
	lch = lerp(lch,(height/2 - 2),_ls);
} else {
	lcw = lerp(lcw,0,_ls);
	lch = lerp(lch,0,_ls);
}

if (lcw > 0.125 && lch > 0.125) {
	var _w = ((lcw + lch) / 2) / (((width/2)-2) + ((height/2)-2)) + 1
	draw_line_width(_cx-(lcw+0.5),_cy-(lch+0.5),_cx+(lcw+1),_cy+(lch+1),_w);
	draw_line_width(_cx-(lcw+0.5),_cy+(lcw+0.5),_cx+(lcw+1),_cy-(lch+1),_w);
}

var _ha = draw_get_halign();
var _va = draw_get_valign();

var _tx = 0;
var _ty = 0;

draw_set_halign(fa_left);
draw_set_valign(fa_middle);
switch (align) {
	case UI_ALIGN.TOP:
		_ty = -height*1.5;
		draw_set_valign(fa_bottom);
	break;
	case UI_ALIGN.BOTTOM:
		_ty = height*1.5;
		draw_set_valign(fa_top);
	break;
	case UI_ALIGN.LEFT:
		_tx = -width*1.5;
		draw_set_halign(fa_right);
	break;
	case UI_ALIGN.RIGHT:
		_tx = width*1.5;
		draw_set_halign(fa_left);
	break;
	case UI_ALIGN.TOP_LEFT:
		_tx = -width*1.5;
		_ty = -height*1.5;
		draw_set_halign(fa_right);
		draw_set_valign(fa_bottom);
	break;
	case UI_ALIGN.TOP_RIGHT:
		_tx = width*1.5;
		_ty = -height*1.5;
		draw_set_halign(fa_left);
		draw_set_valign(fa_bottom);
	break;
	case UI_ALIGN.BOTTOM_LEFT:
		_tx = -width*1.5;
		_ty = height*1.5;
		draw_set_halign(fa_right);
		draw_set_valign(fa_top);
	break;
	case UI_ALIGN.BOTTOM_RIGHT:
		_tx = width*1.5;
		_ty = height*1.5;
		draw_set_halign(fa_left);
		draw_set_valign(fa_top);
	break;
}

draw_text(_cx+_tx,_cy+_ty,text);

draw_set_halign(_ha);
draw_set_valign(_va);