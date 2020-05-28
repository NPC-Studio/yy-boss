/// @description Insert description here
// You can write your code in this editor

// development
#macro BASE "http://localhost:5000/red-shirt-innoquous/us-central1/api"

// production
// #macro BASE "..."

state = ONLINE_STATES.LOGGED_OUT;
token = "";
refresh_token = "";
expiry = -1;
headers = map(
	["Authorization",""]
);

function setup_expiry(token_data) {
	token = token_data[? "token"];
	refresh_token = token_data[? "refresh_token"];
	expiry = token_data[? "expiry"];
	headers[? "Authorization"] = token;
	var tz = date_get_timezone();
	date_set_timezone(timezone_utc);
	var _diff = expiry - date_current_datetime();
	date_set_timezone(tz);
	alarm[0] = _diff * game_get_speed(gamespeed_fps) * 0.6;
}
function register(name,email,pass) {
	var _req = map(
		["display_name",name],
		["email",email],
		["password",pass]
	);
	state = ONLINE_STATES.WAITING;
	http(BASE + "/auth/register","POST",{
		body: json_encode(_req),
		headers: map(
			["Content-Type","application/json"]
		)
	},function(status,http_status,_cl,_sd,result){
		if (http_status != 200) {
			state = ONLINE_STATES.ERROR;
			return false;
		}
		var _body = json_decode(result);
		var _token_map = _body[? "token"];
		setup_expiry(_token_map);
		state = ONLINE_STATES.LOGGED_IN;
		ds_map_destroy(_body);
	});
	ds_map_destroy(_req);
	
}
function login(email,pass) {
	var _req = map(
		["email",email],
		["password",pass]
	);
	state = ONLINE_STATES.WAITING;
	http(BASE + "/auth/login","POST",{
		body: json_encode(_req),
		headers: map(
			["Content-Type","application/json"]
		)
	},function(status,http_status,_cl,_sd,result){
		//show_message_async(result);
		if (http_status != 200){ 
			state = ONLINE_STATES.ERROR;
			return false;
		}
		var _body = json_decode(result);
		var _token_map = _body[? "token"];
		setup_expiry(_token_map);
		state = ONLINE_STATES.LOGGED_IN;
		show_message(string(ONLINE_STATES.LOGGED_IN) + " - " + string(object_index.state) + " - " + string(state))
		ds_map_destroy(_body);
		
	});
}
function refresh(_refresh_token) {
	var _req = map(
		["refresh_token",_refresh_token]
	);
	http(BASE + "/auth/refresh-token","POST",{
		body: json_encode(_req),
		headers: map(
			["Content-Type","application/json"]
		)
	},function(status,http_status,_cl,_sd,result){
		if (http_status != 200){ 
			state = ONLINE_STATES.ERROR;
			return false;
		}
		var _body = json_decode(result);
		var _token_map = _body[? "token"];
		setup_expiry(_token_map);
		ds_map_destroy(_body);
		
	});
}


if (file_exists("lastlogin")) {
	var _buff = buffer_load("lastlogin");
	refresh_token = buffer_read(_buff,buffer_text);
}



enum ONLINE_STATES {
	LOGGED_OUT,
	WAITING,
	LOGGED_IN,
	ERROR
}