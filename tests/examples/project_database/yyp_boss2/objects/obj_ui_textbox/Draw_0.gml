/// @description Insert description here
draw_set_font(font);

draw_set_color(c_gray);
draw_line(x,y+height,x+width,y+height);
// if (focused == true) {
	draw_set_color(c_accent);
	var _xc = x + width/2;
	draw_line(_xc - (width/2)*w,y+height,_xc + (width/2)*w,y+height);
// }
draw_set_color(c_forecolor);
draw_set_halign(fa_left);
draw_set_valign(fa_bottom);
if (type == "password") {
	var _str = "";
	for (var i=0;i<string_length(content);i++) {
		_str += maskchar;
	}
	draw_text(x+2,y+height-2,_str);
} else {
	draw_text(x+2,y+height-2,content);
}
if (content == "") {
	draw_set_alpha(0.5);
	draw_text(x+2,y+height-2,label);
	draw_set_alpha(1);
}