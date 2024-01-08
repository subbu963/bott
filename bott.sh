if [ -z "$BOTT_EXECUTABLE_DIR" ]; then
	export BOTT_EXECUTABLE_DIR="$BOTT_DIR"
fi

alias bott_="$BOTT_EXECUTABLE_DIR/bott"

function bott_init() {
	export bott_last_run_executed_code=""
	export bott_last_run_output=""
	export bott_last_run_exit_code=0
	export bott_last_query_response=""
	export bott_last_query_exit_code=0
	export bott_last_debug_response=""
	export bott_last_debug_exit_code=0
	export bott_last_other_response=""
	export bott_last_other_exit_code=0
	export bott_context=""
}
function bott_execute_code() {
	bott_last_run_executed_code=$1
	bott_last_run_output=$(eval "$1" 2>&1)
	bott_last_run_exit_code=$?

	if [ $bott_last_run_exit_code -ne 0 ]; then
		bott_last_run_output="${bott_last_run_output/"(eval):1: "/""}"
	fi
	return $bott_last_run_exit_code
}
function bott_get_distro() {
	local d=""
	if [[ -f /etc/os-release ]]; then
		# On Linux systems
		source /etc/os-release
		d=$ID
	else
		# On systems other than Linux (e.g. Mac or FreeBSD)
		d=$(uname)
	fi

	case $d in
	"raspbian")
		echo Raspbian
		;;
	"fedora")
		echo Fedora
		;;
	"ubuntu")
		echo Ubuntu
		;;
	"Darwin")
		echo macOS
		;;
	esac
}
function bott_get_shell() {
	if [ -z "${SHELL}" ]; then
		_SHELL=$(ps -o args= -p "$$" | cut -d- -f2)
	else
		_SHELL=$SHELL
	fi
	echo "$_SHELL"
}
function bott!() {
	local subcommand=$1
	case $subcommand in
	"run")
		local code_to_exec="${*/"run"/""}"
		bott_execute_code $code_to_exec
		echo $bott_last_run_output
		return "$bott_last_run_exit_code"
		;;
	"query")
		local distro="$(bott_get_distro)"
		local shell="$(bott_get_shell)"
		local query="${*/"query"/""}"
		local code_to_exec="bott_ query -d \"$distro\" -s \"$shell\" -q \"$query\""
		bott_last_query_response=$(eval "$code_to_exec")
		bott_last_query_exit_code=$?
		if [ $bott_last_query_exit_code -ne 0 ]; then
			echo "$bott_last_query_response"
			return $bott_last_query_exit_code
		fi
		local answer=$(echo "$bott_last_query_response" | awk -v RS="<ANSWER>" -v ORS="" 'NR>1{gsub(/<\/ANSWER>.*/, ""); print}')
		local context=$(echo "$bott_last_query_response" | awk -v RS="<CONTEXT>" -v ORS="" 'NR>1{gsub(/<\/CONTEXT>.*/, ""); print}')
		if [ -z $answer ]; then
			echo "Didnt get your question. Please try asking only questions related to bash commands"
			return 1
		fi
		bott_context="$context"
		echo "Answer: $answer"
		if bott_ confirm -q "Do you want to run the command?"; then
			bott_execute_code $answer
			echo $bott_last_run_output
			return "$bott_last_run_exit_code"
		fi
		;;
	"clear")
		bott_init
		echo "session cleared"
		;;
	"debug")
		local distro="$(bott_get_distro)"
		local shell="$(bott_get_shell)"
		local code_to_exec="bott_ debug -d \"$distro\" -s \"$shell\""
		bott_last_debug_response=$(eval "$code_to_exec")
		bott_last_debug_exit_code=$?
		if [ $bott_last_debug_exit_code -ne 0 ]; then
			echo "Didnt get your question. Please try asking only questions related to bash commands"
			return 1
		fi
		local answer=$(echo "$bott_last_debug_response" | awk -v RS="<ANSWER>" -v ORS="" 'NR>1{gsub(/<\/ANSWER>.*/, ""); print}')
		echo "Answer: $answer"
		;;
	"config")
		local code_to_exec="bott_ $*"
		bott_last_other_response=$(eval "$code_to_exec" 2>&1)
		bott_last_other_exit_code=$?
		echo "$bott_last_other_response"
		local query="${*/"config"/""}"
		if [ $bott_last_other_exit_code -eq 0 ] && [[ ! "$query" =~ ^\ get ]]; then
			bott_init
			echo "config set and session cleared"
		fi

		return $bott_last_other_exit_code
		;;
	*)
		local code_to_exec="bott_ $*"
		bott_last_other_response=$(eval "$code_to_exec" 2>&1)
		bott_last_other_exit_code=$?
		echo "$bott_last_other_response"
		return $bott_last_other_exit_code
		;;
	esac
}
bott_init
