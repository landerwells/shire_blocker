deepwork() {
  read "hours? > how long? (in hours): "
  read "google_amazon? > block google/amazon? (y/n): "
  read "stocks? > block stocks? (y/n): "
  read "messages? > block messages? (y/n): "

  minutes=$((hours * 60))

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

  [[ "$stocks" == "y" ]] && shire start "finance" --lock "$minutes"
  [[ "$google_amazon" == "y" ]] && shire start "google, amazon" --lock "$minutes"
  [[ "$messages" == "y" ]] && shire start "silence" --lock "$minutes"

  ~/.local/bin/arttime --nolearn -a butterfly -t "deep work time – blocking distractions" -g "${hours}h"
}
