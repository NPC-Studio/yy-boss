/// @description Insert description here

draw_set_color(c_accent);
draw_rectangle(x,y,x+width,y+height,false);
draw_set_halign(fa_center);
draw_set_valign(fa_middle);

draw_set_color(c_forecolor);
draw_text(x+width/2,y+height/2,text);