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

  // Handles to various event descriptions and instances
  //TODO these should be held on the Rust side, not here as a bit of JS scope
  let explosionDescription = {}; // Global Event Description for the explosion event.  This event is played as a one-shot and released immediately after it has been created.
  let loopingAmbienceInstance = {}; // Global Event Instance for the looping ambience event.  A single instance is started or stopped based on user input.
  let cancelInstance = {}; // Global Event Instance for the cancel event.  This instance is started and if already playing, restarted.

  // Simple error checking function for all FMOD return values. Can only be used once FMOD runtime
  // has been initialized.
  function CHECK_RESULT(result) {
    //FIXME should have some better error logging here?
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

      // Get the Looping Ambience event
      var loopingAmbienceDescription = {};
      CHECK_RESULT(
        gSystem.getEvent("event:/Ambience/Country", loopingAmbienceDescription)
      );
      console.log("loopingAmbienceDescription", loopingAmbienceDescription);

      CHECK_RESULT(
        loopingAmbienceDescription.val.createInstance(loopingAmbienceInstance)
      );

      // Get the 4 Second Surge event
      var cancelDescription = {};
      CHECK_RESULT(gSystem.getEvent("event:/UI/Cancel", cancelDescription));

      CHECK_RESULT(cancelDescription.val.createInstance(cancelInstance));

      // Get the Explosion event
      CHECK_RESULT(
        gSystem.getEvent("event:/Weapons/Explosion", explosionDescription)
      );

      // Start loading explosion sample data and keep it in memory
      CHECK_RESULT(explosionDescription.val.loadSampleData());

      // Once the loading is finished, re-enable the disabled buttons.
      document.getElementById("playEvent0").disabled = false;
      document.getElementById("playEvent1").disabled = false;
      document.getElementById("playEvent2").disabled = false;
      document.getElementById("playEvent3").disabled = false;

      initState = 3;

      return FMOD.OK;
    },
  };

  // A convenience wrapper for the FMOD object that provides a friendlier API.
  // We use Rust naming conventions for function names since we expect to expose this directly
  // to Rust via wasm-bindgen.
  let fmodController = {
    is_loaded: function () {
      return initState == 3;
    },
    // Update fmod to actually play audio
    tick: function () {
      let result = {};
      result = gSystem.update();
      CHECK_RESULT(result);
    },
    play_event: function (soundid) {
      if (!this.is_loaded()) {
        console.log("ignoring playEvent attempt while not loaded", soundid);
        return;
      }

      console.log("controller playEvent attempt", soundid);

      if (soundid == 0) {
        // One-shot event
        let eventInstance = {};
        CHECK_RESULT(explosionDescription.val.createInstance(eventInstance));
        CHECK_RESULT(eventInstance.val.start());

        // Release will clean up the instance when it completes
        CHECK_RESULT(eventInstance.val.release());
      } else if (soundid == 1) {
        CHECK_RESULT(loopingAmbienceInstance.val.start());
      } else if (soundid == 2) {
        CHECK_RESULT(
          loopingAmbienceInstance.val.stop(FMOD.STUDIO_STOP_IMMEDIATE)
        );
      } else if (soundid == 3) {
        CHECK_RESULT(cancelInstance.val.start());
      }
    },
  };

  // begin initializing the fmod controller right away: get Emscripten to load the FMOD API
  // so that Emscripten will call our FMOD object method callbacks
  FMODModule(FMOD);
  initState = 1;

  return fmodController;
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
