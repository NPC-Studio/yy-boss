function Tween() constructor {
	enum TweenTask {
		Delay,
		CallFunction,
		EaseX,
		EaseY,
		Count
	}
	
	tasks = [];
	currentIndex = -1;
	running = false;
	node = undefined;
	timer = 0;
	
	#region Constructing Tweens
	
	static append = function(_task) {
		tasks[array_length(tasks)] = [_task];
		return self;
	}
	
	static join = function(_task) {
		var _currentIndex = array_length(tasks) - 1;
		tasks[_currentIndex][array_length(tasks[_currentIndex])] = _task;
		return self;
	}
	
	static __makeEase = function(_tweenTask, _easeType, _target, _duration) {
		return { isEase: true, type: _tweenTask, easeType: _easeType, startValue: 0, endValue: _target, duration: _duration };
	}
	
	static delay = function(_frames) {
		return { isEase: false, type: TweenTask.Delay, frames: _frames };
	}
	
	static callback = function(_function, _args) {
		return { isEase: false, type: TweenTask.CallFunction, func: _function, args: _args };
	}
	
	static easeX = function(_easeType, _target, _duration) {
		return __makeEase(TweenTask.EaseX, _easeType, _target, _duration);
	}
	
	static easeY = function(_easeType, _target, _duration) {
		return __makeEase(TweenTask.EaseY, _easeType, _target, _duration);
	}
	
	#endregion
	
	#region Control and Progression
	
	static start = function() {
		if (node == undefined) throw "You cannot start a tween which has not been assigned to a node!";
		running = true;	
		__progressTask();
	}
	
	static __enterTask = function(_task) {
		if (_task.isEase) {
			switch (_task.type) {
				case TweenTask.EaseX: _task.startValue = node.position.x; break;
				case TweenTask.EaseY: _task.startValue = node.position.y; break;
			}
		}
	}
	
	static __progressTask = function(_task) {
		++currentIndex;
		timer = 0;
		if (currentIndex >= array_length(tasks)) {
			currentIndex = -1;
			timer = 0;
			running = false;
			return;
		}
		for (var i = 0; i < array_length(tasks[currentIndex]); i++) {
			__enterTask(tasks[currentIndex][i]);
		}
	}
	
	static __update = function() {
		if (running) {
			var _allDone = true;
			var _tasks = tasks[currentIndex];
			for (var i = 0; i < array_length(_tasks); i++) {
				var _task = _tasks[i];
				if (_task.isEase) {
					var _newValue = ease(_task.startValue, _task.endValue, timer, _task.duration, _task.easeType);
					switch (_task.type) {
						case TweenTask.EaseX: node.position.x = _newValue; break;
						case TweenTask.EaseY: node.position.y = _newValue; break;
					}
					++timer;
					if (_newValue != _task.endValue) _allDone = false;
				} else {
					if (_task.type == TweenTask.Delay) {
						if (timer++ < _task.frames) _allDone = false;
					} else if (_task.type == TweenTask.CallFunction) {
						_task.func(_task.args);
						
					}
				}
			}
			if (_allDone) __progressTask();
		}
	}
	
	#endregion
}