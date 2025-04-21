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
  // 0 = not initialized, 1 = waiting for prerun, 2 = prerun complete, 3 = loaded (and ready to play sound)
  let initState = 0;

  // Global 'System' object which has the Studio API functions.
  let gSystem;
  // Global 'SystemCore' object which has the Core API functions.
  let gSystemCore;

  // Simple error checking function for all FMOD return values. Can only be used once FMOD runtime
  // has been initialized.
  function CHECK_RESULT(result) {
    //FIXME should have some better error handling here?
    if (result != FMOD.OK) {
      let msg = "FMOD Error: '" + FMOD.ErrorString(result) + "'";

      alert(msg);

      throw msg;
    }
  }

  // The FMOD object is a global object that is used to interact with the FMOD API; Emscripten
  // will populate this object with the FMOD API when FMODModule is called with it.
  let FMOD = {
    // runs before emscripten runtime is initialized
    preRun: function () {
      console.debug("fmod preRun");

      let folderName = "/";
      let fileName;
      let canRead = true;
      let canWrite = false;

      fileName = ["Master.bank", "Master.strings.bank", "SFX.bank"];

      for (const fileToLoad of banksToLoad) {
        console.debug("fmod: creating preloaded file", fileToLoad);

        FMOD.FS_createPreloadedFile(
          folderName,
          fileToLoad,
          filesPathPrefix + fileToLoad,
          canRead,
          canWrite
        );
      }

      initState = 2;
    },
    // runs after emscripten runtime is initialized and fmod is loaded
    onRuntimeInitialized: function () {
      console.debug("fmod onRuntimeInitialized, this is ", this);

      // A temporary empty object to hold our system
      let outval = {};
      let result;

      console.log("Creating FMOD System object");

      // Create the system and check the result
      result = FMOD.Studio_System_Create(outval);
      CHECK_RESULT(result);

      console.log("grabbing system object from temporary and storing it");

      // Take out our System object
      gSystem = outval.val;

      result = gSystem.getCoreSystem(outval);
      CHECK_RESULT(result);

      gSystemCore = outval.val;

      // Optional.  Setting DSP Buffer size can affect latency and stability.
      // Processing is currently done in the main thread so anything lower than 2048 samples can cause stuttering on some devices.
      console.log("set DSP Buffer size.");
      result = gSystemCore.setDSPBufferSize(2048, 2);
      CHECK_RESULT(result);

      // Optional.  Set sample rate of mixer to be the same as the OS output rate.
      // This can save CPU time and latency by avoiding the automatic insertion of a resampler at the output stage.
      console.log("Set mixer sample rate");
      result = gSystemCore.getDriverInfo(0, null, null, outval, null, null);
      CHECK_RESULT(result);
      result = gSystemCore.setSoftwareFormat(
        outval.val,
        FMOD.SPEAKERMODE_DEFAULT,
        0
      );
      CHECK_RESULT(result);

      console.log("initialize FMOD");

      // 1024 virtual channels
      result = gSystem.initialize(
        1024,
        FMOD.STUDIO_INIT_NORMAL,
        FMOD.INIT_NORMAL,
        null
      );
      CHECK_RESULT(result);

      // Starting up your typical JavaScript application loop
      console.log("initialize Application");

      console.log("Loading banks");
      for (const bankToLoad of banksToLoad) {
        let bankHandle = {};
        CHECK_RESULT(
          gSystem.loadBankFile(
            "/" + bankToLoad,
            FMOD.STUDIO_LOAD_BANK_NORMAL,
            bankHandle
          )
        );
      }

      initState = 3;

      return FMOD.OK;
    },
  };

  // Wrapper class for FMOD Event Description
  class FmodEvent {
    constructor(eventDescription) {
      this.eventDescription = eventDescription;
    }

    create_instance() {
      let instance = {};
      CHECK_RESULT(this.eventDescription.createInstance(instance));
      return new FmodInstance(instance.val);
    }

    load_sample_data() {
      CHECK_RESULT(this.eventDescription.loadSampleData());
    }
  }

  // Wrapper class for FMOD Event Instance
  class FmodInstance {
    constructor(instance) {
      this.instance = instance;
    }

    start() {
      CHECK_RESULT(this.instance.start());
    }

    stop() {
      CHECK_RESULT(this.instance.stop(FMOD.STUDIO_STOP_IMMEDIATE));
    }

    release() {
      CHECK_RESULT(this.instance.release());
    }
  }

  // A convenience wrapper for the FMOD object that provides a friendlier API.
  // We use Rust naming conventions for function names since we expect to expose this directly
  // to Rust via wasm-bindgen.
  let fmodWeb = {
    is_loaded: function () {
      return initState == 3;
    },
    // Update fmod to actually play audio
    tick: function () {
      if (!this.is_loaded()) {
        return;
      }

      let result = {};
      result = gSystem.update();
      CHECK_RESULT(result);
    },
    get_event: function (event_name) {
      if (!this.is_loaded()) {
        console.log("ignoring get_event attempt while not loaded", event_name);
        return null;
      }

      let eventDescription = {};
      CHECK_RESULT(gSystem.getEvent(event_name, eventDescription));
      return new FmodEvent(eventDescription.val);
    },
  };

  // begin initializing the fmod controller right away: get Emscripten to load the FMOD API
  // so that Emscripten will call our FMOD object method callbacks
  FMODModule(FMOD);
  initState = 1;

  return fmodWeb;
}

//==============================================================================
// Example code
//==============================================================================

// // Function called when user presses HTML Play Sound button, with parameter 0, 1 or 2.
// function playEvent(soundid) {
//   console.log("window playEvent attempt", soundid);

//   window.fmodController.playEvent(soundid);
// }

// // Expose playEvent to global scope
// window.playEvent = playEvent;

// Called from main, does some application setup.  In our case we will load some sounds.

// // Called from main, on an interval that updates at a regular rate (like in a game loop).
// // Prints out information, about the system, and importantly calles System::udpate().
// function updateApplication() {
//   var result;
//   var cpu = {};

//   result = gSystemCore.getCPUUsage(cpu);
//   CHECK_RESULT(result);

//   var channelsplaying = {};
//   result = gSystemCore.getChannelsPlaying(channelsplaying, null);
//   CHECK_RESULT(result);

//   document.querySelector("#display_out").value =
//     "Channels Playing = " +
//     channelsplaying.val +
//     " : CPU = dsp " +
//     cpu.dsp.toFixed(2) +
//     "% stream " +
//     cpu.stream.toFixed(2) +
//     "% update " +
//     cpu.update.toFixed(2) +
//     "% total " +
//     (cpu.dsp + cpu.stream + cpu.update).toFixed(2) +
//     "%";

//   var numbuffers = {};
//   var buffersize = {};
//   result = gSystemCore.getDSPBufferSize(buffersize, numbuffers);
//   CHECK_RESULT(result);

//   var rate = {};
//   result = gSystemCore.getSoftwareFormat(rate, null, null);
//   CHECK_RESULT(result);

//   var sysrate = {};
//   result = gSystemCore.getDriverInfo(0, null, null, sysrate, null, null);
//   CHECK_RESULT(result);

//   var ms = (numbuffers.val * buffersize.val * 1000) / rate.val;
//   document.querySelector("#display_out2").value =
//     "Mixer rate = " +
//     rate.val +
//     "hz : System rate = " +
//     sysrate.val +
//     "hz : DSP buffer size = " +
//     numbuffers.val +
//     " buffers of " +
//     buffersize.val +
//     " samples (" +
//     ms.toFixed(2) +
//     " ms)";
// }
