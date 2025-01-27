import mido
import RPi.GPIO as GPIO
import time
from flask import Flask, render_template, request

# Initial configuration
midi_port_name = "f_midi:f_midi 16:0"  # Replace with the exact name of your MIDI port
max_presets = 127  # Maximum number of presets (limited by MIDI)

# GPIO setup for buttons
next_button_pin = 17  # GPIO pin for the "Next" button
previous_button_pin = 27  # GPIO pin for the "Previous" button
custom_button_pin = 22  # GPIO pin for the "Custom" button
GPIO.setmode(GPIO.BCM)
GPIO.setup(next_button_pin, GPIO.IN, pull_up_down=GPIO.PUD_UP)  # Pull-up resistor for "Next" button
GPIO.setup(previous_button_pin, GPIO.IN, pull_up_down=GPIO.PUD_UP)  # Pull-up resistor for "Previous" button
GPIO.setup(custom_button_pin, GPIO.IN, pull_up_down=GPIO.PUD_UP)  # Pull-up resistor for "Custom" button

# Flask app setup
app = Flask(__name__)

# Current state
current_preset = 0  # Start at preset 0
button_hex_values = {
    "next": "C0 01",  # Default hex for "Next"
    "previous": "C0 00",  # Default hex for "Previous"
    "custom": "C0 7F"  # Default hex for "Custom"
}

# Function to send a Program Change or custom hex message
def send_midi_message(hex_message):
    try:
        with mido.open_output(midi_port_name) as port:
            status, data1 = [int(x, 16) for x in hex_message.split()]  # Convert hex to integers
            msg = mido.Message.from_bytes([status, data1])
            port.send(msg)
            print(f"MIDI message sent: {hex_message}")
    except Exception as e:
        print(f"Error while sending MIDI message: {e}")

# Function to go to the next preset
def next_preset():
    global current_preset
    if current_preset < max_presets - 1:
        current_preset += 1
    else:
        print("Already at the last preset.")
    send_midi_message(button_hex_values["next"])

# Function to go to the previous preset
def previous_preset():
    global current_preset
    if current_preset > 0:
        current_preset -= 1
    else:
        print("Already at the first preset.")
    send_midi_message(button_hex_values["previous"])

# Function to send a custom preset
def custom_action():
    send_midi_message(button_hex_values["custom"])

# Flask routes
@app.route('/')
def index():
    return render_template('index.html', hex_values=button_hex_values)

@app.route('/update', methods=['POST'])
def update():
    button = request.form.get('button')
    hex_value = request.form.get('hex_value')
    if button in button_hex_values:
        button_hex_values[button] = hex_value
    return render_template('index.html', hex_values=button_hex_values, message="Hex values updated!")

# Main loop to detect button presses
if __name__ == "__main__":
    import threading

    # Start Flask app in a separate thread
    def run_flask():
        app.run(host='0.0.0.0', port=5000)

    flask_thread = threading.Thread(target=run_flask)
    flask_thread.daemon = True
    flask_thread.start()

    print("MIDI Preset Control - Using GPIO Buttons")
    print("""Press the "Next", "Previous" or "Custom" button to send MIDI messages.""")

    try:
        while True:
            next_pressed = GPIO.input(next_button_pin) == GPIO.LOW  # "Next" button pressed
            previous_pressed = GPIO.input(previous_button_pin) == GPIO.LOW  # "Previous" button pressed
            custom_pressed = GPIO.input(custom_button_pin) == GPIO.LOW  # "Custom" button pressed

            if next_pressed:
                next_preset()
                time.sleep(0.2)  # Debounce delay

            if previous_pressed:
                previous_preset()
                time.sleep(0.2)  # Debounce delay

            if custom_pressed:
                custom_action()
                time.sleep(0.2)  # Debounce delay

            time.sleep(0.05)  # Small delay to reduce CPU usage

    except KeyboardInterrupt:
        print("\nProgram interrupted.")

    finally:
        GPIO.cleanup()  # Reset GPIO pins to a safe state
