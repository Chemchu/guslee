import { motion } from "framer-motion";
import { useState } from "react";
import Typewriter from 'typewriter-effect';
import Card from "../card";

const LandingPage = () => {
    return (
        <FondoImagen />
    );
}

const FondoImagen = () => {
    const [firstWriterDone, setFirstWriter] = useState<boolean>(false);
    const [secondWriterDone, setSecondWriter] = useState<boolean>(false);
    const [thirdWriterDone, setThirdWriter] = useState<boolean>(false);

    return (
        <main className="overflow-x-hidden text-white font-outfit-light h-screen w-screen">
            <div className="bg-main bg-cover bg-no-repeat bg-center w-full h-full cursor-default">
                <div className="transform flex justify-around items-center text-base sm:text-lg xl:text-xl">
                    <div className="transition hover:scale-150 hover:duration-100 cursor-pointer">
                        Sobre mí
                    </div>

                    <div className="transition hover:scale-150 hover:duration-100 cursor-pointer">
                        Trabajos
                    </div>

                    <div className="transition hover:scale-150 hover:duration-100 cursor-pointer">
                        Twitter
                    </div>

                    <div className="transition hover:scale-150 hover:duration-100 cursor-pointer">
                        Instagram
                    </div>

                    <div className="transition hover:scale-150 hover:duration-100 cursor-pointer">
                        GitHub
                    </div>
                </div>
                <hr className="my-2 animate-pulse" />
                <div className="flex flex-col mt-10 pl-10 lg:mt-0 lg:pl-6 lg:justify-center w-full h-full">
                    <div className="flex text-xl sm:text-4xl xl:text-6xl">
                        <div className="font-outfit mr-2 md:mr-4">
                            {!firstWriterDone ?
                                <Typewriter
                                    options={{ delay: 60 }}
                                    onInit={(typewriter) => {
                                        typewriter.typeString('¡Hola!')
                                            .stop()
                                            .callFunction(() => { setFirstWriter(true) })
                                            .start();
                                    }}
                                />
                                :
                                '¡Hola!'
                            }

                        </div>
                        <div className="font-outfit-thin">
                            {firstWriterDone && !secondWriterDone ?
                                <Typewriter
                                    options={{ delay: 55 }}
                                    onInit={(typewriter) => {
                                        typewriter.typeString('Soy Gus')
                                            .pauseFor(200)
                                            .deleteAll()
                                            .typeString('Me llamo Gustavo Lee')
                                            .stop()
                                            .callFunction(() => { setSecondWriter(true) })
                                            .start();
                                    }}
                                />
                                :
                                secondWriterDone && 'Me llamo Gustavo Lee'
                            }
                        </div>
                    </div>
                    <div className="text-lg sm:text-3xl xl:text-5xl my-6">
                        <div className="flex gap-2 font-outfit-thin mr-2 md:mr-4">
                            {/* Segundo */}
                            {secondWriterDone && !thirdWriterDone ?
                                <Typewriter
                                    options={{ delay: 40 }}
                                    onInit={(typewriter) => {
                                        typewriter.typeString('Soy')
                                            .pauseFor(400)
                                            .stop()
                                            .callFunction(() => { setThirdWriter(true) })
                                            .start();
                                    }}
                                />
                                :
                                thirdWriterDone && 'Soy '
                            }
                            {thirdWriterDone &&
                                <Typewriter
                                    options={{ delay: 70, deleteSpeed: 27, loop: true }}
                                    onInit={(typewriter) => {
                                        typewriter
                                            .typeString('ingeniero de software')
                                            .pauseFor(1200)
                                            .deleteChars(22)
                                            .typeString('guitarrista')
                                            .pauseFor(700)
                                            .deleteChars(11)
                                            .typeString('brasileño')
                                            .pauseFor(800)
                                            .deleteChars(9)
                                            .typeString('informático')
                                            .pauseFor(900)
                                            .deleteChars(12)
                                            .typeString('fullstack developer')
                                            .pauseFor(1200)
                                            .deleteChars(19)
                                            .typeString('panadero')
                                            .pauseFor(500)
                                            .deleteChars(8)
                                            .typeString('gracioso')
                                            .pauseFor(1200)
                                            .deleteChars(8)
                                            .typeString('carismático')
                                            .pauseFor(950)
                                            .deleteChars(11)
                                            .stop()
                                            .start();
                                    }}
                                />
                            }
                        </div>
                    </div>
                </div>
            </div >

            <div className="w-full h-full bg-orange-500">
                <h1>
                    Sobre mí
                </h1>
            </div>

            <div className="w-full h-full bg-green-600">
                <h1>
                    Trabajos
                </h1>
            </div>

            <div className="w-full h-full bg-cyan-600">
                <h1>
                    Twitter
                </h1>
            </div>

            <div className="w-full h-full bg-pink-700">
                <h1>
                    Instagram
                </h1>
            </div>

            <div className="w-full h-full bg-slate-600">
                <h1>
                    GitHub
                </h1>
            </div>

        </main>
    );
}

export default LandingPage;