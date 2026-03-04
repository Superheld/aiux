// System-Monitor: Nur bei Auffaelligkeiten an Cortex weiterleiten.
// Normalbetrieb → nur loggen, kein Forward.
let d = parse_json(data);

// Schwellwerte
let cpu_warn = 80.0;
let mem_warn = 90.0;
let temp_warn = 70.0;

let warnings = [];

if d.cpu_percent > cpu_warn {
    warnings.push(`CPU ${d.cpu_percent}%`);
}
if d.mem_percent > mem_warn {
    warnings.push(`RAM ${d.mem_percent}%`);
}
if d.temp_celsius != () && d.temp_celsius > temp_warn {
    warnings.push(`Temp ${d.temp_celsius}°C`);
}

if warnings.len() > 0 {
    let text = `[nerve/system] Warnung: ${warnings}`;
    #{ forward: true, target: "neocortex", text: text }
} else {
    #{ forward: false }
}
