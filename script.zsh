deepwork() {
  read "hours? > how long? (in hours): "
  read "google_amazon? > block google/amazon? (y/n): "
  read "stocks? > block stocks? (y/n): "
  read "messages? > block messages? (y/n): "

  minutes=$((hours * 60))
  blocker="/Applications/Cold Turkey Blocker.app/Contents/MacOS/Cold Turkey Blocker"

  to_block=()
  [[ "$stocks" == "y" ]] && to_block+=("finance")
  [[ "$google_amazon" == "y" ]] && to_block+=("google, amazon")
  [[ "$messages" == "y" ]] && to_block+=("silence")

  echo ""
  echo "blocking ${to_block[*]} for $hours hours."
  echo "press any key to cancel..."

  for i in {10..1}; do
    echo -n "$i... "
    read -t 1 -n 1 key && { echo "cancelled."; return; }
  done

  echo ""

  [[ "$stocks" == "y" ]] && "$blocker" -start "finance" -lock "$minutes"
  [[ "$google_amazon" == "y" ]] && "$blocker" -start "google, amazon" -lock "$minutes"
  [[ "$messages" == "y" ]] && "$blocker" -start "silence" -lock "$minutes"

  ~/.local/bin/arttime --nolearn -a butterfly -t "deep work time – blocking distractions" -g "${hours}h"
}

shire # will list out all available subcommands

shire block list # list all available blocks
shire block start "Algorithmic feeds" --lock duration
shire block stop "Algorithmic feeds" # turns off the block if it isn't locked

shire service start # runs install and starts daemon
shire service stop # stops the daemon if there is one running
shire service restart
shire service install # installs needed things like plist and 





shire debug

shire config # Probably puts users into a configuration menu creating a default config for shire

shire schedule list # prints out a textual version of the schedule?

