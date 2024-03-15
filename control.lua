local stats
local timer = 0

script.on_event(defines.events.on_tick, function(event)
  local new_stats = remote.call("Ultracube", "better-victory-screen-statistics").by_force.player.ultracube.stats

  if not stats then
    stats = new_stats
    return
  end

  if new_stats["cube-utilisation"].value > stats["cube-utilisation"].value then
    game.write_file("buttplug.commands", "1.0")
    timer = 60
  end

  if new_stats["cubes-consumed-total"].value > stats["cubes-consumed-total"].value then
    game.write_file("buttplug.commands", "1.0")
    timer = 60
  end

  if new_stats["cubes-reconstructed"].value > stats["cubes-reconstructed"].value then
    game.write_file("buttplug.commands", "0.5")
    timer = 60
  end

  if new_stats["cube-distance-travelled"].value > stats["cube-distance-travelled"].value then
    game.write_file("buttplug.commands", "0.1")
    timer = 30
  end

  if timer > 0 then
    timer = timer - 1
    if timer == 0 then
      game.write_file("buttplug.commands", "0.0")
    end
  end

  stats = new_stats
end)
