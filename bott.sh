alias bott_="/Users/aditya/RustroverProjects/bott/target/debug/bott"

function bott_init() {
	export bott_last_executed_code=""
	export bott_last_output=""
	export bott_last_response=""
	export bott_last_exit_code=0
	export bott_context=""
}
function bott_execute_code() {
	bott_last_executed_code=$1
	bott_last_output=$(eval "$1" 2>&1)
	bott_last_exit_code=$?

	[ $bott_last_exit_code -ne 0 ]
	bott_last_output="${bott_last_output/"(eval):1: "/""}"
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
		echo $bott_last_output
		return "$bott_last_exit_code"
		;;
	"query")
		local distro="$(bott_get_distro)"
		local shell="$(bott_get_shell)"
		local query="${*/"query"/""}"
		local code_to_exec="bott_ query -d \"$distro\" -s \"$shell\" -q \"$query\""
		local res=$(eval "$code_to_exec")
		bott_last_response="$res"
		if [ $? -ne 0 ]; then
			echo "Didnt get your question. Please try asking only questions related bash commands"
			return 1
		fi
		local answer=$(echo "$res" | awk -v RS="<ANSWER>" -v ORS="" 'NR>1{gsub(/<\/ANSWER>.*/, ""); print}')
		local context=$(echo "$res" | awk -v RS="<CONTEXT>" -v ORS="" 'NR>1{gsub(/<\/CONTEXT>.*/, ""); print}')
		bott_context="$context"
		echo "Answer: $answer"
		if bott_ confirm -q "Do you want to run the command?"; then
			bott_execute_code $answer
			echo $bott_last_output
			return "$bott_last_exit_code"
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
		local res=$(eval "$code_to_exec")
		if [ $? -ne 0 ]; then
			echo "Didnt get your question. Please try asking only questions related bash commands"
			return 1
		fi
		local answer=$(echo "$res" | awk -v RS="<ANSWER>" -v ORS="" 'NR>1{gsub(/<\/ANSWER>.*/, ""); print}')
		echo "Answer: $answer"
		;;
	*)
		echo "Unknown command"
		;;
	esac
}
bott_init
