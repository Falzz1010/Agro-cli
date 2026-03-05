const plantsGrid = document.getElementById('plants-grid');
const connectionStatus = document.getElementById('connection-status');
const plantCountEl = document.getElementById('plant-count');
const hubUptimeEl = document.getElementById('hub-uptime');

let plants = {};
let startTime = Date.now();
let moistureChart;
let tempHumidityChart;
let historyChart;
let currentHistoryHours = 24;

// Initialize Chart.js with explicit line rendering
function initChart() {
    const ctx = document.getElementById('moisture-chart').getContext('2d');
    moistureChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: [],
            datasets: []
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            animation: false,
            interaction: {
                mode: 'index',
                intersect: false
            },
            scales: {
                y: {
                    beginAtZero: true,
                    max: 100,
                    grid: {
                        color: 'rgba(255, 255, 255, 0.1)',
                        drawBorder: true,
                        lineWidth: 1
                    },
                    ticks: { color: '#a0a0a0' }
                },
                x: {
                    grid: {
                        display: false,
                        drawBorder: true
                    },
                    ticks: {
                        color: '#a0a0a0',
                        maxRotation: 0,
                        autoSkip: true,
                        maxTicksLimit: window.innerWidth < 600 ? 5 : 12
                    }
                }
            },
            plugins: {
                legend: {
                    display: true,
                    labels: {
                        color: '#f1f1f1',
                        font: { family: 'Outfit' },
                        usePointStyle: true
                    }
                }
            }
        }
    });

    const ctx2 = document.getElementById('temp-humidity-chart').getContext('2d');
    tempHumidityChart = new Chart(ctx2, {
        type: 'line',
        data: { labels: [], datasets: [] },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            animation: false,
            interaction: {
                mode: 'index',
                intersect: false
            },
            scales: {
                y: {
                    type: 'linear',
                    display: true,
                    position: 'left',
                    title: { display: true, text: 'Temp (°C)', color: '#8892b0', font: { family: 'Outfit', size: 10 } },
                    grid: {
                        color: 'rgba(255,255,255,0.05)',
                        drawBorder: true,
                        lineWidth: 1
                    },
                    ticks: { color: '#a0a0a0' }
                },
                y1: {
                    type: 'linear',
                    display: true,
                    position: 'right',
                    title: { display: true, text: 'Humidity (%)', color: '#8892b0', font: { family: 'Outfit', size: 10 } },
                    grid: {
                        drawOnChartArea: false,
                        drawBorder: true
                    },
                    ticks: { color: '#a0a0a0' }
                },
                x: {
                    grid: {
                        display: false,
                        drawBorder: true
                    },
                    ticks: { color: '#a0a0a0' }
                }
            },
            plugins: {
                legend: {
                    display: true,
                    position: 'top',
                    labels: {
                        boxWidth: 12,
                        padding: 10,
                        color: '#f1f1f1',
                        font: { family: 'Outfit', size: 11 },
                        usePointStyle: true
                    }
                }
            }
        }
    });

    const ctx3 = document.getElementById('history-chart').getContext('2d');
    historyChart = new Chart(ctx3, {
        type: 'line',
        data: { labels: [], datasets: [] },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            animation: false,
            interaction: {
                mode: 'index',
                intersect: false
            },
            scales: {
                y: {
                    type: 'linear',
                    display: true,
                    position: 'left',
                    title: { display: true, text: 'Moisture & Humidity (%)', color: '#8892b0' },
                    max: 100,
                    grid: {
                        color: 'rgba(255,255,255,0.05)',
                        drawBorder: true,
                        lineWidth: 1
                    },
                    ticks: { color: '#a0a0a0' }
                },
                y1: {
                    type: 'linear',
                    display: true,
                    position: 'right',
                    title: { display: true, text: 'Temp (°C)', color: '#8892b0' },
                    grid: {
                        drawOnChartArea: false,
                        drawBorder: true
                    },
                    ticks: { color: '#a0a0a0' }
                },
                x: {
                    grid: {
                        display: false,
                        drawBorder: true
                    },
                    ticks: {
                        color: '#a0a0a0',
                        maxTicksLimit: window.innerWidth < 600 ? 6 : 12,
                        maxRotation: 0,
                        autoSkip: true
                    }
                }
            },
            plugins: {
                legend: {
                    display: true,
                    labels: {
                        color: '#f1f1f1',
                        font: { family: 'Outfit' },
                        usePointStyle: true
                    }
                }
            }
        }
    });
}

initChart();

// Auto-update uptime every second
setInterval(() => {
    const diff = Math.floor((Date.now() - startTime) / 1000);
    const mins = Math.floor(diff / 60);
    const secs = diff % 60;
    hubUptimeEl.innerText = `${mins}m ${secs}s`;
}, 1000);

function connect() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const socket = new WebSocket(`${protocol}//${window.location.host}/ws`);

    socket.onopen = () => {
        connectionStatus.innerText = 'Live: Connected';
        connectionStatus.parentElement.querySelector('.pulse').style.background = '#4ade80';
        connectionStatus.parentElement.querySelector('.pulse').style.boxShadow = '0 0 0 0 rgba(74, 222, 128, 0.7)';
    };

    socket.onmessage = (event) => {
        const msg = JSON.parse(event.data);
        if (msg.type === 'SensorUpdate') {
            updateDashboard(msg.data);
        } else if (msg.type === 'AiLog') {
            handleAiLog(msg.data);
        }
    };

    socket.onclose = () => {
        connectionStatus.innerText = 'Live: Disconnected';
        connectionStatus.parentElement.querySelector('.pulse').style.background = '#ef4444';
        connectionStatus.parentElement.querySelector('.pulse').style.boxShadow = 'none';

        // Reconnect after 3 seconds
        setTimeout(connect, 3000);
    };

    socket.onerror = (err) => {
        console.error('Socket error:', err);
        socket.close();
    };
}

function updateDashboard(data) {
    // Remove empty state if present
    const emptyState = document.querySelector('.empty-state');
    if (emptyState) emptyState.remove();

    const name = data.plant_name;

    if (!plants[name]) {
        createPlantCard(data);
        addChartDataset(name);
        addPlantToSelect(name);
    }

    plants[name] = data;
    updatePlantCard(data);
    updateSensorCharts(data);

    if (window.debugAgro) {
        console.log(`[DASHBOARD] Updated ${name}:`, data);
    }

    // Update plant count
    plantCountEl.innerText = Object.keys(plants).length;
}

function handleAiLog(log) {
    const container = document.getElementById('ai-log-container');
    const emptyLog = container.querySelector('.empty-log');
    if (emptyLog) emptyLog.remove();

    const entry = document.createElement('div');
    entry.className = 'ai-entry';
    entry.innerHTML = `
        <div class="meta">
            <span>AgroAI Agent</span>
            <span>${log.timestamp}</span>
        </div>
        <div class="query">👤 ${log.query}</div>
        <div class="response">🤖 ${log.response}</div>
    `;

    container.prepend(entry);

    // Keep only last 20 logs
    if (container.children.length > 20) {
        container.removeChild(container.lastChild);
    }
}

function addChartDataset(name) {
    const colors = ['#4ade80', '#3b82f6', '#8b5cf6', '#f59e0b', '#ec4899'];
    const colorIndex = moistureChart.data.datasets.length % colors.length;
    const color = colors[colorIndex];

    console.log(`Adding dataset for ${name} with color ${color}`);

    const moistureData = new Array(moistureChart.data.labels.length).fill(null);
    const tempData = new Array(tempHumidityChart.data.labels.length).fill(null);
    const humidData = new Array(tempHumidityChart.data.labels.length).fill(null);

    // Moisture Dataset
    moistureChart.data.datasets.push({
        label: name,
        data: moistureData,
        borderColor: color,
        backgroundColor: color + '33',
        fill: true,
        borderWidth: 3,
        tension: 0.4,
        pointRadius: 4,
        pointHoverRadius: 6,
        pointBackgroundColor: color,
        pointBorderColor: '#1a1a1a',
        pointBorderWidth: 2,
        spanGaps: true
    });

    // Temp Dataset
    tempHumidityChart.data.datasets.push({
        label: `${name} (Temp)`,
        data: tempData,
        borderColor: color,
        backgroundColor: 'transparent',
        borderWidth: 3,
        tension: 0.4,
        yAxisID: 'y',
        pointRadius: 4,
        pointHoverRadius: 6,
        pointBackgroundColor: color,
        pointBorderColor: '#1a1a1a',
        pointBorderWidth: 2,
        spanGaps: true
    });

    // Humidity Dataset
    tempHumidityChart.data.datasets.push({
        label: `${name} (Humid)`,
        data: humidData,
        borderColor: color,
        borderDash: [5, 5],
        backgroundColor: 'transparent',
        borderWidth: 2,
        tension: 0.4,
        yAxisID: 'y1',
        pointRadius: 3,
        pointHoverRadius: 5,
        pointBackgroundColor: color,
        pointBorderColor: '#1a1a1a',
        pointBorderWidth: 1,
        spanGaps: true
    });

    moistureChart.update('none');
    tempHumidityChart.update('none');
}

function updateSensorCharts(data) {
    const now = data.timestamp;

    console.log(`Updating charts with data:`, data);

    // 1. Update Moisture Chart
    if (!moistureChart.data.labels.includes(now)) {
        moistureChart.data.labels.push(now);
        moistureChart.data.datasets.forEach(ds => {
            if (ds.label === data.plant_name) {
                ds.data.push(data.moisture);
            } else {
                ds.data.push(null);
            }
        });
    } else {
        const labelIndex = moistureChart.data.labels.indexOf(now);
        const dataset = moistureChart.data.datasets.find(ds => ds.label === data.plant_name);
        if (dataset) dataset.data[labelIndex] = data.moisture;
    }

    // 2. Update Temp/Humidity Chart
    if (!tempHumidityChart.data.labels.includes(now)) {
        tempHumidityChart.data.labels.push(now);
        tempHumidityChart.data.datasets.forEach(ds => {
            if (ds.label === `${data.plant_name} (Temp)`) {
                ds.data.push(data.temperature);
            } else if (ds.label === `${data.plant_name} (Humid)`) {
                ds.data.push(data.humidity);
            } else {
                ds.data.push(null);
            }
        });
    } else {
        const labelIndex = tempHumidityChart.data.labels.indexOf(now);
        const dsTemp = tempHumidityChart.data.datasets.find(ds => ds.label === `${data.plant_name} (Temp)`);
        const dsHumid = tempHumidityChart.data.datasets.find(ds => ds.label === `${data.plant_name} (Humid)`);
        if (dsTemp) dsTemp.data[labelIndex] = data.temperature;
        if (dsHumid) dsHumid.data[labelIndex] = data.humidity;
    }

    // Keep charts synced (last 30 points)
    if (moistureChart.data.labels.length > 30) {
        moistureChart.data.labels.shift();
        moistureChart.data.datasets.forEach(ds => ds.data.shift());
    }
    if (tempHumidityChart.data.labels.length > 30) {
        tempHumidityChart.data.labels.shift();
        tempHumidityChart.data.datasets.forEach(ds => ds.data.shift());
    }

    console.log(`Moisture chart datasets:`, moistureChart.data.datasets.length, `labels:`, moistureChart.data.labels.length);
    console.log(`Temp chart datasets:`, tempHumidityChart.data.datasets.length, `labels:`, tempHumidityChart.data.labels.length);

    moistureChart.update('none');
    tempHumidityChart.update('none');
}

function createPlantCard(data) {
    const card = document.createElement('div');
    card.className = 'plant-card';
    card.id = `plant-${data.plant_name.replace(/\s+/g, '-')}`;
    card.innerHTML = `
        <span class="plant-type">Healthy</span>
        <h2>${data.plant_name}</h2>
        <div class="sensor-controls">
            <div class="sensor-item">
                <label>Moisture</label>
                <div class="value moisture-value">--%</div>
            </div>
            <div class="sensor-item">
                <label>Temp</label>
                <div class="value temp-value">--°C</div>
            </div>
            <div class="sensor-item">
                <label>Humidity</label>
                <div class="value humidity-value">--%</div>
            </div>
            <div class="sensor-item">
                <label>Last Updated</label>
                <div class="value time-value">--:--:--</div>
            </div>
        </div>
        <div class="progress-container">
            <div class="progress-bar moisture-bar"></div>
        </div>
        <div class="plant-controls">
            <button class="btn btn-water" onclick="handleWater(event, '${data.plant_name}')">💧 Water Now</button>
            <button class="btn" onclick="handleSettings(event, '${data.plant_name}')">⚙️</button>
            <button class="btn btn-delete" onclick="handleDelete(event, '${data.plant_name}')">🗑️</button>
        </div>
    `;
    plantsGrid.appendChild(card);
}

function handleWater(event, name) {
    console.log(`Watering ${name}...`);
    // Optimistic UI
    const btn = event.currentTarget;
    const oldText = btn.innerText;
    btn.innerText = '🚿 Watering...';
    btn.disabled = true;

    fetch('/api/command/water', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ plant_name: name })
    })
        .then(res => res.json())
        .then(data => {
            setTimeout(() => {
                btn.innerText = '✅ Done';
                setTimeout(() => {
                    btn.innerText = oldText;
                    btn.disabled = false;
                }, 2000);
            }, 1000);
        })
        .catch(err => {
            console.error(err);
            btn.innerText = '❌ Error';
            btn.disabled = false;
        });
}

function handleDelete(event, name) {
    if (!confirm(`⚠️ Are you sure you want to permanently delete '${name}' and all its sensor data?`)) {
        return;
    }

    const btn = event.currentTarget;
    const oldText = btn.innerHTML;
    btn.innerHTML = '⏳';
    btn.disabled = true;

    fetch('/api/command/delete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ plant_name: name })
    })
        .then(res => res.json())
        .then(data => {
            if (data.status === 'deleted') {
                // Remove card
                const card = document.getElementById(`plant-${name.replace(/\s+/g, '-')}`);
                if (card) card.remove();

                // Remove from state
                delete plants[name];
                plantCountEl.innerText = Object.keys(plants).length;

                // Remove from select
                const select = document.getElementById('history-plant-select');
                for (let i = 0; i < select.options.length; i++) {
                    if (select.options[i].value === name) {
                        select.remove(i);
                        break;
                    }
                }
            } else {
                throw new Error(data.message || 'Delete failed');
            }
        })
        .catch(err => {
            console.error(err);
            btn.innerHTML = '❌';
            setTimeout(() => {
                btn.innerHTML = oldText;
                btn.disabled = false;
            }, 2000);
        });
}

// Handle Export CSV action
function handleExportCsv() {
    const plantName = document.getElementById('history-plant-select').value;
    if (!plantName) {
        alert("Please select a plant from the dropdown first to export its data.");
        return;
    }

    // Direct browser to the download endpoint, which will prompt auth if needed.
    window.location.href = `/api/export/${encodeURIComponent(plantName)}`;
}

// Modal Elements
const settingsModal = document.getElementById('settings-modal');
const settingsForm = document.getElementById('settings-form');
const modalPlantName = document.getElementById('modal-plant-name');
const settingName = document.getElementById('setting-name');
const settingMoisture = document.getElementById('setting-moisture');
const settingWater = document.getElementById('setting-water');

function handleSettings(event, name) {
    const data = plants[name];
    if (!data) return;

    modalPlantName.innerText = `Settings: ${name}`;
    settingName.value = name;

    // Fallback to defaults or rules (should ideally come from backend rules if not set)
    settingMoisture.value = data.min_moisture || 40;
    settingWater.value = data.water_ml || 200;

    settingsModal.classList.add('active');
}

function closeModal() {
    settingsModal.classList.remove('active');
}

// Close modal on outside click
window.onclick = (event) => {
    if (event.target == settingsModal) {
        closeModal();
    }
};

settingsForm.onsubmit = async (e) => {
    e.preventDefault();

    const name = settingName.value;
    const min_moisture = parseFloat(settingMoisture.value);
    const water_ml = parseInt(settingWater.value);

    const saveBtn = settingsForm.querySelector('button[type="submit"]');
    const oldText = saveBtn.innerText;
    saveBtn.innerText = '💾 Saving...';
    saveBtn.disabled = true;

    try {
        const response = await fetch('/api/command/settings', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ plant_name: name, min_moisture, water_ml })
        });

        if (response.ok) {
            saveBtn.innerText = '✅ Saved';
            // Update local state optimisticially or wait for next broadcast
            if (plants[name]) {
                plants[name].min_moisture = min_moisture;
                plants[name].water_ml = water_ml;
            }

            setTimeout(() => {
                closeModal();
                saveBtn.innerText = oldText;
                saveBtn.disabled = false;
            }, 1000);
        } else {
            throw new Error('Failed to save');
        }
    } catch (err) {
        console.error(err);
        saveBtn.innerText = '❌ Error';
        saveBtn.disabled = false;
        setTimeout(() => { saveBtn.innerText = oldText; }, 2000);
    }
};

function updatePlantCard(data) {
    const card = document.getElementById(`plant-${data.plant_name.replace(/\s+/g, '-')}`);
    if (!card) return;

    const moistureVal = card.querySelector('.moisture-value');
    const tempVal = card.querySelector('.temp-value');
    const humidityVal = card.querySelector('.humidity-value');
    const timeVal = card.querySelector('.time-value');
    const moistureBar = card.querySelector('.moisture-bar');

    moistureVal.innerText = `${data.moisture.toFixed(1)}%`;
    tempVal.innerText = `${data.temperature.toFixed(1)}°C`;
    humidityVal.innerText = `${data.humidity.toFixed(1)}%`;
    timeVal.innerText = data.timestamp;

    // Update progress bar
    moistureBar.style.width = `${data.moisture}%`;

    // Status color based on REAL-TIME moisture and CUSTOM threshold
    const statusLabel = card.querySelector('.plant-type');
    const threshold = data.min_moisture || 40;

    if (data.moisture < threshold) {
        statusLabel.innerText = 'Needs Water';
        statusLabel.style.color = '#ef4444';
        statusLabel.style.background = 'rgba(239, 68, 68, 0.1)';
        moistureBar.style.background = 'linear-gradient(to right, #ef4444, #f97316)';
    } else {
        statusLabel.innerText = 'Healthy';
        statusLabel.style.color = '#4ade80';
        statusLabel.style.background = 'rgba(74, 222, 128, 0.1)';
        moistureBar.style.background = 'linear-gradient(to right, #4ade80, #3b82f6)';
    }
}

// --- Sensor History Logic ---

const historyPlantSelect = document.getElementById('history-plant-select');
const historyEmpty = document.getElementById('history-empty');

historyPlantSelect.addEventListener('change', (e) => {
    const plantName = e.target.value;
    if (plantName) {
        loadHistory(plantName, currentHistoryHours);
    } else {
        historyChart.data.labels = [];
        historyChart.data.datasets = [];
        historyChart.update();
        historyEmpty.style.display = 'block';
    }
});

function addPlantToSelect(name) {
    // Only add if not already present
    for (let i = 0; i < historyPlantSelect.options.length; i++) {
        if (historyPlantSelect.options[i].value === name) return;
    }
    const opt = document.createElement('option');
    opt.value = name;
    opt.innerText = name;
    historyPlantSelect.appendChild(opt);
}

function setHistoryPeriod(hours) {
    currentHistoryHours = hours;

    // Update button styling
    document.querySelectorAll('.btn-period').forEach(btn => {
        btn.classList.remove('active');
        if (parseInt(btn.dataset.hours) === hours) {
            btn.classList.add('active');
        }
    });

    const plantName = historyPlantSelect.value;
    if (plantName) {
        loadHistory(plantName, hours);
    }
}

async function loadHistory(plantName, hours) {
    historyEmpty.style.display = 'none';

    try {
        const res = await fetch(`/api/history/${encodeURIComponent(plantName)}?hours=${hours}`);

        if (!res.ok) {
            console.error(`Fetch failed with status ${res.status}`);
            historyChart.data.labels = [];
            historyChart.data.datasets = [];
            historyChart.update();
            historyEmpty.innerText = `Error: ${res.status === 401 ? "Unauthorized. Please refresh page." : "HTTP " + res.status}`;
            historyEmpty.style.display = 'block';
            return;
        }

        const result = await res.json();

        if (result.error) {
            console.error("Server reported history error:", result.message);
            historyChart.data.labels = [];
            historyChart.data.datasets = [];
            historyChart.update();
            historyEmpty.innerText = `Error: ${result.message}`;
            historyEmpty.style.display = 'block';
            return;
        }

        if (!result.data || result.data.length === 0) {
            historyChart.data.labels = [];
            historyChart.data.datasets = [];
            historyChart.update();
            historyEmpty.innerText = `No data found for ${plantName} in the last ${hours} hours.`;
            historyEmpty.style.display = 'block';
            return;
        }

        const data = result.data;

        // Prepare datasets
        const labels = data.map(d => {
            // Format "YYYY-MM-DD HH:MM:SS" -> "MM-DD HH:MM" or "HH:MM"
            const parts = d.timestamp.split(' ');
            if (hours <= 24 && parts.length === 2) {
                return parts[1].substring(0, 5); // Just HH:MM
            }
            return d.timestamp.substring(5, 16); // MM-DD HH:MM
        });

        const moistureData = data.map(d => d.moisture);
        const humidityData = data.map(d => d.humidity);
        const tempData = data.map(d => d.temperature);

        console.log(`Loading history for ${plantName}: ${data.length} points`);

        historyChart.data.labels = labels;
        historyChart.data.datasets = [
            {
                label: 'Moisture (%)',
                data: moistureData,
                borderColor: '#4ade80',
                backgroundColor: 'rgba(74, 222, 128, 0.15)',
                fill: true,
                borderWidth: 3,
                tension: 0.2,
                yAxisID: 'y',
                pointRadius: 3,
                pointHoverRadius: 6,
                pointBackgroundColor: '#4ade80',
                pointBorderColor: '#1a1a1a',
                pointBorderWidth: 2,
                spanGaps: true
            },
            {
                label: 'Humidity (%)',
                data: humidityData,
                borderColor: '#3b82f6',
                borderDash: [5, 5],
                backgroundColor: 'transparent',
                borderWidth: 2,
                tension: 0.2,
                yAxisID: 'y',
                pointRadius: 3,
                pointHoverRadius: 5,
                pointBackgroundColor: '#3b82f6',
                pointBorderColor: '#1a1a1a',
                pointBorderWidth: 1,
                spanGaps: true
            },
            {
                label: 'Temperature (°C)',
                data: tempData,
                borderColor: '#f59e0b',
                backgroundColor: 'transparent',
                borderWidth: 3,
                tension: 0.2,
                yAxisID: 'y1',
                pointRadius: 3,
                pointHoverRadius: 6,
                pointBackgroundColor: '#f59e0b',
                pointBorderColor: '#1a1a1a',
                pointBorderWidth: 2,
                spanGaps: true
            }
        ];

        historyChart.update();

    } catch (e) {
        console.error("Failed to load history:", e);
        historyEmpty.innerText = "Error loading historical data.";
        historyEmpty.style.display = 'block';
    }
}

connect();
