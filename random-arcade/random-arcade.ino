#include <Arduino.h>
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>
#include <Adafruit_MPU6050.h>
#include <Adafruit_Sensor.h>

// The four production sketches are embedded unchanged in separate namespaces.
// Their setup()/loop() names are renamed only to avoid collisions with the
// launcher and with each other.
namespace Pong {
void rearmShakeDetection();
#define setup pongSetup
#define loop pongLoop
#include "../pong/pong.ino"
#undef setup
#undef loop
}

namespace Snake {
#define setup snakeSetup
#define loop snakeLoop
#include "../snake/snake.ino"
#undef setup
#undef loop
}

namespace Shooter {
float enemyFireChancePercent();
void applyScorePenalty(int amount);
#define setup shooterSetup
#define loop shooterLoop
#include "../shooter/shooter.ino"
#undef setup
#undef loop
}

namespace TiltRacer {
#define setup racerSetup
#define loop racerLoop
#include "../tilt-racer/tilt-racer.ino"
#undef setup
#undef loop
}

enum GameId { PONG_GAME, SNAKE_GAME, SHOOTER_GAME, RACER_GAME };
GameId activeGame;
int previousGame = -1;
bool activeRound = false;

void selectAndStartGame() {
  int selected = random(0, 4);
  while (selected == previousGame) selected = random(0, 4);
  previousGame = selected;
  activeGame = (GameId)selected;

  if (activeGame == PONG_GAME) Pong::pongSetup();
  if (activeGame == SNAKE_GAME) Snake::snakeSetup();
  if (activeGame == SHOOTER_GAME) Shooter::shooterSetup();
  if (activeGame == RACER_GAME) TiltRacer::racerSetup();

  activeRound = false;
}

bool gameIsWaiting() {
  if (activeGame == PONG_GAME) return Pong::waitingForShake;
  if (activeGame == SNAKE_GAME) return !Snake::started;
  if (activeGame == SHOOTER_GAME) return Shooter::waitingForShake;
  return TiltRacer::waitingForShake;
}

void runActiveGame() {
  if (activeGame == PONG_GAME) Pong::pongLoop();
  if (activeGame == SNAKE_GAME) Snake::snakeLoop();
  if (activeGame == SHOOTER_GAME) Shooter::shooterLoop();
  if (activeGame == RACER_GAME) TiltRacer::racerLoop();
}

void setup() {
  randomSeed((unsigned long)esp_random());
  selectAndStartGame();
}

void loop() {
  runActiveGame();

  // Each real sketch owns its own start/game-over flow. Once it returns to
  // that sketch's shake screen after a played round, select a new game.
  if (!activeRound && !gameIsWaiting()) activeRound = true;
  if (activeRound && gameIsWaiting()) {
    delay(100);
    selectAndStartGame();
  }
}
