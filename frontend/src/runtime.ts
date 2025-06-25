import { AnimationManager } from './animation-manager';
import { Display } from './display';
import { MessageQueue } from './message-queue';
import { Renderer } from './renderer';
import { GameController, BotPlayer } from './controller';

export interface RuntimeModule {
  init(): boolean;
}

class Runtime {
  public display: Display;
  public renderer: Renderer;
  public messageQueue: MessageQueue;
  public animationManager: AnimationManager;
  public gameController: GameController | null = null;

  private modules: RuntimeModule[];

  public constructor() {
    this.messageQueue = new MessageQueue();
    this.animationManager = new AnimationManager();
    this.display = new Display();
    this.renderer = new Renderer();
    this.modules = [
      this.animationManager,
      this.display,
      this.renderer,
      this.messageQueue,
    ];
  }

  public addModule(module: RuntimeModule): void {
    this.modules.push(module);
  }

  public init(): boolean {
    for (const module of this.modules) {
      if (!module.init()) {
        return false;
      }
    }

    this.gameController = new GameController(
      new BotPlayer(), // White player
      new BotPlayer(), // Black player
    );

    return true;
  }

  // public tick(): void {
  //   for (const module of this.modules) {
  //     module.tick();
  //   }
  // }
}

export const runtime = new Runtime();
