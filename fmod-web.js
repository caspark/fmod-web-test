// A wrapper around the FMOD HTML5 API that makes it more ergonomic to use from Rust.

/**
 * Initialize FMOD.
 *
 * @param {string} filesPathPrefix - The URL path prefix of all banks to load - e.g. `/assets/`
 * @param {string[]} banksToLoad - An array of strings, each representing a bank to load. E.g.
 * `["Master.bank", "Master.strings.bank", "SFX.bank"]`
 * @returns {Object} - A controller object for interacting with FMOD.
 */
export default function (filesPathPrefix, banksToLoad) {
  // 0 = waiting for prerun, 1 = prerun complete, 2 = loaded (and ready to play sound)
  let initState = 0;

  // The actual emscripten-instantiated FMOD object.
  let FMOD;

  // Global 'System' object which has the Studio API functions.
  let gSystemStudio;
  // Global 'SystemCore' object which has the Core API functions.
  let gSystemCore;

  let gFmodWebBackend;

  // Simple error checking function for all FMOD return values. Can only be used once FMOD runtime
  // has been initialized.
  function CHECK_RESULT(result) {
    if (result != FMOD.OK) {
      let msg = "FMOD Error: '" + FMOD.ErrorString(result) + "'";
      console.error(msg);
      throw msg;
    }
  }

  function check_finite_vec2(vec, name) {
    // check not nan
    if (isNaN(vec[0]) || isNaN(vec[1])) {
      throw "vec2." + name + " has NaN: " + vec[0] + ", " + vec[1];
    }
    // check not infinite
    if (
      vec[0] == Infinity ||
      vec[0] == -Infinity ||
      vec[1] == Infinity ||
      vec[1] == -Infinity
    ) {
      throw "vec2." + name + " has infinite: " + vec[0] + ", " + vec[1];
    }
  }

  function build3DAttrs(position, velocity) {
    // Our coordinates are 10 pixels to 1 meter and x runs to the right, so we need to handle that.
    // Theoretically, we should be able to just set the core system's 3d settings, but seems like
    // sounds are still attenuated too much that way.
    const UP_DIR = -1; // Assuming gravity points down (+y), so up is -y

    check_finite_vec2(position, "position");
    check_finite_vec2(velocity, "velocity");

    let attributes3d = FMOD._3D_ATTRIBUTES();
    attributes3d.position = FMOD.VECTOR();
    attributes3d.position.x = -position[0] / 10.0;
    attributes3d.position.y = position[1] / 10.0;
    attributes3d.position.z = 0.0;
    attributes3d.velocity = FMOD.VECTOR();
    attributes3d.velocity.x = -velocity[0] / 10.0;
    attributes3d.velocity.y = velocity[1] / 10.0;
    attributes3d.velocity.z = 0.0;
    attributes3d.forward = FMOD.VECTOR();
    attributes3d.forward.x = 0.0;
    attributes3d.forward.y = 0.0;
    attributes3d.forward.z = 1.0;
    attributes3d.up = FMOD.VECTOR();
    attributes3d.up.x = 0.0;
    attributes3d.up.y = UP_DIR;
    attributes3d.up.z = 0.0;
    return attributes3d;
  }

  class FmodWebBackend {
    constructor(masterBank) {
      if (!masterBank) {
        throw (
          "Can't create FmodWebBackend with null/undefined masterBank: " +
          masterBank
        );
      }
      this.masterBank = masterBank;
    }

    update() {
      CHECK_RESULT(gSystemStudio.update());
    }

    shutdown() {
      CHECK_RESULT(gSystemStudio.release());
    }

    get_event(event_name) {
      let eventDescriptionOut = {};
      CHECK_RESULT(gSystemStudio.getEvent(event_name, eventDescriptionOut));
      return new FmodEventDescription(eventDescriptionOut.val);
    }

    event_count() {
      let countOut = {};
      CHECK_RESULT(this.masterBank.getEventCount(countOut));
      return countOut.val;
    }

    get_event_list() {
      let count = this.event_count();

      let arrayOut = { val: new Array(count) };
      let countOut = {};

      // Call the FMOD function with proper parameters
      CHECK_RESULT(this.masterBank.getEventList(arrayOut, count, countOut));
      if (countOut.val != count) {
        throw "FMOD event list count mismatch";
      }

      let result = [];
      // Process each event description in the array
      for (const eventDesc of arrayOut.val) {
        result.push(new FmodEventDescription(eventDesc));
      }

      return result;
    }

    set_listeners(listenersIn) {
      // make sure we don't exceed the max number of listeners
      let listeners = listenersIn.slice(0, FMOD.MAX_LISTENERS);

      for (let listenerIdx = 0; listenerIdx < listeners.length; listenerIdx++) {
        let listener = listeners[listenerIdx];

        let attributes3d = build3DAttrs(listener.position, listener.velocity);
        let attenuationPosition = null;
        CHECK_RESULT(
          gSystemStudio.setListenerAttributes(
            listenerIdx,
            attributes3d,
            attenuationPosition
          )
        );

        CHECK_RESULT(
          gSystemStudio.setListenerWeight(listenerIdx, listener.weight)
        );
      }
    }

    set_parameter_by_name(name, value) {
      let ignoreSeekSpeed = false;
      CHECK_RESULT(
        gSystemStudio.setParameterByName(name, value, ignoreSeekSpeed)
      );
    }
  }

  // Wrapper class for FMOD Event Description
  class FmodEventDescription {
    constructor(eventDescription) {
      if (!eventDescription) {
        throw (
          "Can't create FmodEventDescription with null/undefined eventDescription: " +
          eventDescription
        );
      }
      this.eventDescription = eventDescription;
    }

    create_instance() {
      let instanceOut = {};
      CHECK_RESULT(this.eventDescription.createInstance(instanceOut));
      return new FmodEventInstance(instanceOut.val);
    }

    get_path() {
      let pathOut = {};
      let pathOutLength = 4096; // probably doesn't matter?
      let retrievedOut = {};
      CHECK_RESULT(
        this.eventDescription.getPath(pathOut, pathOutLength, retrievedOut)
      );
      if (retrievedOut.val == pathOutLength) {
        throw "FMOD event path name too long: " + pathOut.val;
      }

      return pathOut.val;
    }

    load_sample_data() {
      CHECK_RESULT(this.eventDescription.loadSampleData());
    }
  }

  // Wrapper class for FMOD Event Instance
  class FmodEventInstance {
    constructor(instance) {
      if (!instance) {
        throw (
          "Can't create FmodEventInstance with null/undefined instance: " +
          instance
        );
      }
      this.instance = instance;
    }

    release() {
      CHECK_RESULT(this.instance.release());
    }

    start() {
      CHECK_RESULT(this.instance.start());
    }

    stop() {
      CHECK_RESULT(this.instance.stop(FMOD.STUDIO_STOP_IMMEDIATE));
    }

    set_3d_attributes(position, velocity) {
      let attributes3d = build3DAttrs(position, velocity);
      CHECK_RESULT(this.instance.set3DAttributes(attributes3d));
    }

    get_playback_state() {
      let stateOut = {};
      CHECK_RESULT(this.instance.getPlaybackState(stateOut));

      if (stateOut.val == FMOD.STUDIO_PLAYBACK_PLAYING) {
        return "Playing";
      } else if (stateOut.val == FMOD.STUDIO_PLAYBACK_SUSTAINING) {
        return "Sustaining";
      } else if (stateOut.val == FMOD.STUDIO_PLAYBACK_STOPPED) {
        return "Stopped";
      } else if (stateOut.val == FMOD.STUDIO_PLAYBACK_STARTING) {
        return "Starting";
      } else if (stateOut.val == FMOD.STUDIO_PLAYBACK_STOPPING) {
        return "Stopping";
      } else {
        throw "Unknown FMOD playback state: " + stateOut.val;
      }
    }
  }

  // The FMOD object is a global object that is used to interact with the FMOD API; Emscripten
  // will populate this object with the FMOD API when FMODModule is called with it.
  FMOD = {
    // runs before emscripten runtime is initialized
    preRun: function () {
      let folderName = "/";
      let canRead = true;
      let canWrite = false;

      for (const fileToLoad of banksToLoad) {
        FMOD.FS_createPreloadedFile(
          folderName,
          fileToLoad,
          filesPathPrefix + fileToLoad,
          canRead,
          canWrite
        );
      }

      initState = 1;
    },
    // runs after emscripten runtime is initialized and fmod is loaded
    onRuntimeInitialized: function () {
      // A temporary empty object to hold return values
      let outval = {};

      console.log("Creating FMOD Studio System");

      // Create the systems
      CHECK_RESULT(FMOD.Studio_System_Create(outval));
      gSystemStudio = outval.val;
      CHECK_RESULT(gSystemStudio.getCoreSystem(outval));
      gSystemCore = outval.val;

      // Optional.  Setting DSP Buffer size can affect latency and stability.
      // Processing is currently done in the main thread so anything lower than 2048 samples can cause stuttering on some devices.
      console.log("FMOD setting DSP buffer size to 2048, 2");
      CHECK_RESULT(gSystemCore.setDSPBufferSize(2048, 2));

      // Optional.  Set sample rate of mixer to be the same as the OS output rate.
      // This can save CPU time and latency by avoiding the automatic insertion of a resampler at the output stage.
      console.log("FMOD setting mixer sample rate to OS output rate");
      CHECK_RESULT(
        gSystemCore.getDriverInfo(0, null, null, outval, null, null)
      );
      CHECK_RESULT(
        gSystemCore.setSoftwareFormat(outval.val, FMOD.SPEAKERMODE_DEFAULT, 0)
      );

      console.log("FMOD initializing Studio");
      // 1024 virtual channels
      CHECK_RESULT(
        gSystemStudio.initialize(
          1024,
          FMOD.STUDIO_INIT_NORMAL,
          FMOD.INIT_NORMAL,
          null
        )
      );

      console.log("FMOD loading banks", banksToLoad);
      let masterBank;
      for (const bankToLoad of banksToLoad) {
        let bankOut = {};
        CHECK_RESULT(
          gSystemStudio.loadBankFile(
            "/" + bankToLoad,
            FMOD.STUDIO_LOAD_BANK_NORMAL,
            bankOut
          )
        );
        if (bankToLoad == "Master.bank") {
          CHECK_RESULT(bankOut.val.loadSampleData());

          masterBank = bankOut.val;
        }
        // (we just throw away the bank handle here because we don't support unloading banks)
      }

      if (!masterBank) {
        throw "Master bank not found: at least one bank filename must be 'Master.bank'";
      }

      console.log("FMOD loading complete");
      initState = 2;

      gFmodWebBackend = new FmodWebBackend(masterBank);

      return FMOD.OK;
    },
  };

  // A convenience wrapper for the FMOD object that provides a friendlier API.
  // We use Rust naming conventions for function names since we expect to expose this directly
  // to Rust via wasm-bindgen.
  let fmodLoader = {
    get_loaded: function () {
      //NB: we haven't got any particular detection of error states here - so if audio loading is
      //failing completely, we'll just hang in whatever state we were in when the error occurred.
      //(Ideally we'd save the latest error and throw it here again so it can be 'caught' as a Rust
      // error type)
      if (initState == 0) {
        throw "Waiting for prerun";
      } else if (initState == 1) {
        throw "Setting up fmod";
      } else if (initState == 2) {
        return gFmodWebBackend;
      } else {
        // this is a bug: we should have enumerated all known steps of the loading process above.
        throw "Unknown FMOD loading state";
      }
    },
  };

  // begin initializing the fmod controller right away: get Emscripten to load the FMOD API
  // so that Emscripten will call our FMOD object method callbacks
  FMODModule(FMOD);

  return fmodLoader;
}
