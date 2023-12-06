_SHELL=""
set_shell(){
  if [ -z "${SHELL}" ]; then
    _SHELL=$(ps -o args= -p "$$" | cut -d- -f2)
  else
    _SHELL=$SHELL
  fi
  return
}

set_shell

alias bott="/Users/aditya/RustroverProjects/bott/target/debug/bott"

export bott_last_executed_code=""
export bott_last_output=""
export bott_last_exit_code=0

function bott_execute_code() {
  bott_last_executed_code=$1
  bott_last_output=$(eval "$1" 2>&1)
  bott_last_exit_code=$?

  [ $bott_last_exit_code -ne 0 ]; bott_last_output="${bott_last_output/"(eval):1: "/""}"
}
function b!() {
    local subcommand=$1
    case $subcommand in
      "run")
        local code_to_exec="${*/"run"/""}"
        code_to_exec="${code_to_exec##*( )}"
        bott_execute_code $code_to_exec
        echo $bott_last_output
        return "$bott_last_exit_code"
        ;;
      "query")
        local query="${*/"query"/""}"
        local code_to_exec="bott query \"$query\""
        local x=$(eval "$code_to_exec")
        echo "$x"
#              local in
#              read in
#              echo you said $in
              ;;
      *)
        echo "Unknown command"
      ;;
    esac
}